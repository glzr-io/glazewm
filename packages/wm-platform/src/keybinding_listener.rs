use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use tokio::sync::mpsc;
use wm_common::KeybindingConfig;

use crate::{
  find_longest_match, parse_key_binding, platform_event::KeybindingEvent,
  platform_impl, Dispatcher, Key,
};

#[derive(Debug, Clone)]
pub struct ActiveKeybinding {
  pub keys: Vec<Key>,
  pub config: KeybindingConfig,
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
  keybinding_map: Arc<Mutex<HashMap<Key, Vec<ActiveKeybinding>>>>,

  /// The underlying keyboard hook used to listen for key events.
  keyboard_hook: platform_impl::KeyboardHook,
}

impl KeybindingListener {
  /// Creates an instance of `KeybindingListener`.
  pub fn new(
    dispatcher: Dispatcher,
    keybindings: &[KeybindingConfig],
  ) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let keybinding_map =
      Arc::new(Mutex::new(Self::create_keybinding_map(keybindings)));

    let keyboard_hook = Self::create_keyboard_hook(
      dispatcher,
      keybinding_map.clone(),
      event_tx,
    )?;

    Ok(Self {
      event_rx,
      keybinding_map,
      keyboard_hook,
    })
  }

  /// Creates and starts the keyboard hook with the callback.
  fn create_keyboard_hook(
    dispatcher: Dispatcher,
    keybinding_map: Arc<Mutex<HashMap<Key, Vec<ActiveKeybinding>>>>,
    event_tx: mpsc::UnboundedSender<KeybindingEvent>,
  ) -> crate::Result<platform_impl::KeyboardHook> {
    platform_impl::KeyboardHook::new(
      dispatcher,
      move |event: platform_impl::KeyEvent| -> bool {
        if !event.is_keypress {
          return false;
        }

        let Ok(keybinding_map) = keybinding_map.lock() else {
          tracing::error!("Failed to acquire lock on keybinding map.");
          return false;
        };

        // Find trigger key candidates.
        if let Some(candidates) = keybinding_map.get(&event.key) {
          // Convert to the format expected by find_longest_match.
          let candidate_tuples: Vec<_> = candidates
            .iter()
            .map(|binding| (binding.keys.clone(), binding))
            .collect();

          if let Some(active_binding) =
            find_longest_match(&candidate_tuples, event.key, |key| {
              event.is_key_down(key)
            })
          {
            let _ = event_tx
              .send(KeybindingEvent(active_binding.config.clone()));
            return true;
          }
        }

        false
      },
    )
  }

  /// Builds the keybinding map from configs.
  fn create_keybinding_map(
    keybindings: &[KeybindingConfig],
  ) -> HashMap<Key, Vec<ActiveKeybinding>> {
    let mut keybinding_map = HashMap::new();

    for config in keybindings {
      for binding in &config.bindings {
        match parse_key_binding(binding) {
          Ok(keys) => {
            if let Some(&trigger_key) = keys.last() {
              keybinding_map
                .entry(trigger_key)
                .or_insert_with(Vec::new)
                .push(ActiveKeybinding {
                  keys,
                  config: config.clone(),
                });
            }
          }
          Err(err) => {
            tracing::warn!(
              "Failed to parse keybinding '{}': {}",
              binding,
              err
            );
          }
        }
      }
    }

    keybinding_map
  }

  /// Returns the next keybinding event from the listener.
  ///
  /// This method will block until a keybinding event is available.
  pub async fn next_event(&mut self) -> Option<KeybindingEvent> {
    self.event_rx.recv().await
  }

  /// Updates the keybindings for the keybinding listener.
  ///
  /// # Panics
  ///
  /// If the internal mutex is poisoned.
  pub fn update(&self, keybindings: &[KeybindingConfig]) {
    *self.keybinding_map.lock().unwrap() =
      Self::create_keybinding_map(keybindings);
  }

  /// Stops the keybinding listener.
  pub fn stop(&mut self) -> crate::Result<()> {
    self.keyboard_hook.stop()
  }
}

impl Drop for KeybindingListener {
  fn drop(&mut self) {
    let _ = self.stop();
  }
}
