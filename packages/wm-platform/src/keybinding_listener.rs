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

/// Listener for system-wide keybindings.
pub struct KeybindingListener {
  event_rx: mpsc::UnboundedReceiver<KeybindingEvent>,
}

impl KeybindingListener {
  /// Creates a new keybinding listener using the provided dispatcher.
  pub fn new(
    dispatcher: Dispatcher,
    keybindings: &[KeybindingConfig],
  ) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    // Build keybinding map.
    let keybinding_map = Self::build_keybinding_map(keybindings);
    let keybinding_map = Arc::new(Mutex::new(keybinding_map));

    let callback = move |event: platform_impl::KeyEvent| -> bool {
      if !event.is_keypress {
        return false;
      }

      let Ok(keybinding_map) = keybinding_map.lock() else {
        tracing::error!("Failed to acquire lock on keybinding map.");
        return false;
      };

      // Find trigger key candidates
      if let Some(candidates) = keybinding_map.get(&event.key) {
        // Convert to the format expected by find_longest_match
        let candidate_tuples: Vec<_> = candidates
          .iter()
          .map(|binding| (binding.keys.clone(), binding))
          .collect();

        if let Some(active_binding) =
          find_longest_match(&candidate_tuples, event.key, |key| {
            event.is_key_down(key)
          })
        {
          let _ =
            event_tx.send(KeybindingEvent(active_binding.config.clone()));
          return true;
        }
      }

      false
    };

    // Create and start the keyboard hook with the callback.
    dispatcher.dispatch_sync(move || {
      let keyboard_hook = platform_impl::KeyboardHook::new(callback)?;

      // TODO: Avoid use of `std::mem::forget`.
      std::mem::forget(keyboard_hook);

      crate::Result::Ok(())
    })?;

    Ok(Self { event_rx })
  }

  /// Builds the keybinding map from configs.
  fn build_keybinding_map(
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

  /// Stops the keybinding listener.
  pub fn stop(&mut self) {
    todo!()
  }
}
