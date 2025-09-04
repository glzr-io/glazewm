use tokio::sync::mpsc;
use wm_common::KeybindingConfig;

use crate::{platform_event::KeybindingEvent, platform_impl, Dispatcher};

/// Listener for system-wide keybindings.
pub struct KeybindingListener {
  event_rx: mpsc::UnboundedReceiver<KeybindingEvent>,
  // _keyboard_hook: Option<std::sync::Arc<platform_impl::KeyboardHook>>,
}

impl KeybindingListener {
  /// Creates a new keybinding listener using the provided dispatcher.
  pub fn new(
    dispatcher: Dispatcher,
    keybindings: &Vec<KeybindingConfig>,
  ) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    // Create and start the keyboard hook.
    let keybindings = keybindings.clone();
    dispatcher.dispatch_sync(move || {
      let mut keyboard_hook =
        platform_impl::KeyboardHook::new(&keybindings, event_tx)?;

      keyboard_hook.start()?;

      std::mem::forget(keyboard_hook);
      crate::Result::Ok(())
    });

    Ok(Self {
      event_rx,
      // _keyboard_hook: Some(keyboard_hook),
    })
  }

  /// Returns the next keybinding event from the listener.
  ///
  /// This method will block until a keybinding event is available.
  pub async fn next_event(&mut self) -> Option<KeybindingEvent> {
    self.event_rx.recv().await
  }
}
