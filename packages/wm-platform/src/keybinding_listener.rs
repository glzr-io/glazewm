use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
};

use tokio::sync::mpsc;

use crate::{
  platform_event::KeybindingEvent, platform_impl, Dispatcher, Key,
};

const MODIFIER_KEYS: [Key; 8] = [
  Key::LShift,
  Key::RShift,
  Key::LCtrl,
  Key::RCtrl,
  Key::LAlt,
  Key::RAlt,
  Key::LCmd,
  Key::RCmd,
];

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Keybinding(Vec<Key>);

impl Keybinding {
  /// Creates a new keybinding from a vector of keys.
  ///
  /// # Errors
  ///
  /// Returns an error if the keybinding is empty.
  pub fn new(keys: Vec<Key>) -> crate::Result<Self> {
    if keys.is_empty() {
      return Err(crate::Error::InvalidKeybinding);
    }

    Ok(Self(keys))
  }

  /// Returns the keys in the keybinding.
  #[must_use]
  pub fn keys(&self) -> &[Key] {
    &self.0
  }

  /// Returns the trigger key in the keybinding.
  #[must_use]
  #[allow(clippy::missing_panics_doc)]
  pub fn trigger_key(&self) -> &Key {
    // SAFETY: Keys vector is verified to be non-empty in
    // `Keybinding::new`.
    self.0.last().unwrap()
  }
}

/// A listener for system-wide keybindings.
#[derive(Debug)]
pub struct KeybindingListener {
  /// A receiver channel for outgoing keybinding events.
  event_rx: mpsc::UnboundedReceiver<KeybindingEvent>,

  /// A map of keybindings to their trigger key.
  ///
  /// The trigger key is the final key in a keybinding. For example,
  /// in the keybinding `[Key::Cmd, Key::Shift, Key::A]`, `Key::A` is the
  /// trigger key.
  keybinding_map: Arc<Mutex<HashMap<Key, Vec<Keybinding>>>>,

  /// Whether the listener is currently enabled.
  enabled: Arc<AtomicBool>,

  /// The underlying keyboard hook used to listen for key events.
  keyboard_hook: platform_impl::KeyboardHook,
}

impl KeybindingListener {
  /// Creates an instance of `KeybindingListener`.
  pub fn new(
    keybindings: &[Keybinding],
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let keybinding_map =
      Arc::new(Mutex::new(Self::create_keybinding_map(keybindings)));

    let enabled = Arc::new(AtomicBool::new(true));

    let keyboard_hook = Self::create_keyboard_hook(
      keybinding_map.clone(),
      enabled.clone(),
      event_tx,
      dispatcher,
    )?;

    Ok(Self {
      event_rx,
      keybinding_map,
      enabled,
      keyboard_hook,
    })
  }

  /// Creates and starts the keyboard hook with the given callback.
  fn create_keyboard_hook(
    keybinding_map: Arc<Mutex<HashMap<Key, Vec<Keybinding>>>>,
    enabled: Arc<AtomicBool>,
    event_tx: mpsc::UnboundedSender<KeybindingEvent>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<platform_impl::KeyboardHook> {
    platform_impl::KeyboardHook::new(
      move |event: platform_impl::KeyEvent| -> bool {
        if !enabled.load(Ordering::Relaxed) || !event.is_keypress {
          return false;
        }

        let Ok(keybinding_map) = keybinding_map.lock() else {
          tracing::error!("Failed to acquire lock on keybinding map.");
          return false;
        };

        // Find keybinding candidates whose trigger key is the pressed key.
        // TODO: This can probably be simplified.
        if let Some(candidates) = keybinding_map.get(&event.key) {
          let mut cached_key_states = HashMap::new();

          // Find the matching keybindings based on the pressed keys.
          let matched_keybindings =
            candidates.iter().filter(|keybinding| {
              keybinding.keys().iter().all(|&key| {
                if key == event.key {
                  return true;
                }

                if let Some(&is_key_down) = cached_key_states.get(&key) {
                  return is_key_down;
                }

                let is_key_down = event.is_key_down(key);
                cached_key_states.insert(key, is_key_down);
                is_key_down
              })
            });

          // Find the longest matching keybinding.
          let Some(longest_keybinding) = matched_keybindings
            .max_by_key(|keybinding| keybinding.keys().len())
          else {
            return false;
          };

          // Get the modifier keys to reject based on the longest matching
          // keybinding.
          let mut modifier_keys_to_reject =
            MODIFIER_KEYS.iter().filter(|&&modifier_key| {
              !longest_keybinding.keys().contains(&modifier_key)
                && !longest_keybinding
                  .keys()
                  .contains(&Self::generic_modifier_key(modifier_key))
            });

          // Check if any modifier keys to reject are currently down.
          let has_modifier_keys_to_reject =
            modifier_keys_to_reject.any(|&modifier_key| {
              if let Some(&is_key_down) =
                cached_key_states.get(&modifier_key)
              {
                is_key_down
              } else {
                event.is_key_down(modifier_key)
              }
            });

          if has_modifier_keys_to_reject {
            return false;
          }

          let _ =
            event_tx.send(KeybindingEvent(longest_keybinding.clone()));

          return true;
        }

        false
      },
      dispatcher,
    )
  }

  /// Builds the keybinding map from configs.
  fn create_keybinding_map(
    keybindings: &[Keybinding],
  ) -> HashMap<Key, Vec<Keybinding>> {
    let mut keybinding_map = HashMap::new();

    for keybinding in keybindings {
      keybinding_map
        .entry(*keybinding.trigger_key())
        .or_insert_with(Vec::new)
        .push(keybinding.clone());
    }

    keybinding_map
  }

  /// Gets the generic modifier key for a given key.
  fn generic_modifier_key(key: Key) -> Key {
    match key {
      Key::LCmd | Key::RCmd => Key::Cmd,
      Key::LCtrl | Key::RCtrl => Key::Ctrl,
      Key::LAlt | Key::RAlt => Key::Alt,
      Key::LShift | Key::RShift => Key::Shift,
      // TODO: Not ideal, shouldn't panic if used incorrectly.
      _ => unreachable!(),
    }
  }

  /// Returns the next keybinding event from the listener.
  ///
  /// This will block until a keybinding event is available.
  pub async fn next_event(&mut self) -> Option<KeybindingEvent> {
    self.event_rx.recv().await
  }

  /// Updates the keybindings for the keybinding listener.
  ///
  /// # Panics
  ///
  /// If the internal mutex is poisoned.
  pub fn update(&self, keybindings: &[Keybinding]) {
    *self.keybinding_map.lock().unwrap() =
      Self::create_keybinding_map(keybindings);
  }

  /// Enables or disables the keybinding listener.
  pub fn enable(&mut self, enabled: bool) {
    self.enabled.store(enabled, Ordering::Relaxed);
  }

  /// Terminates the keybinding listener.
  pub fn terminate(&mut self) -> crate::Result<()> {
    self.keyboard_hook.terminate()
  }
}

impl Drop for KeybindingListener {
  fn drop(&mut self) {
    let _ = self.terminate();
  }
}
