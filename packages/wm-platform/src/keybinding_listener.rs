use tokio::sync::mpsc;

use crate::{platform_event::KeybindingEvent, platform_impl, Dispatcher};

/// Listener for system-wide keybindings.
pub struct KeybindingListener {
  event_rx: mpsc::UnboundedReceiver<KeybindingEvent>,
  _keyboard_hook: Option<std::sync::Arc<platform_impl::KeyboardHook>>,
}

impl KeybindingListener {
  /// Creates a new keybinding listener using the provided dispatcher.
  pub fn new(dispatcher: Dispatcher) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    // TODO: Get keybindings from config through the dispatcher.
    // For now, create an empty keybindings list.
    let keybindings = vec![];

    // Create and start the keyboard hook>
    let keyboard_hook =
      platform_impl::KeyboardHook::new(&keybindings, event_tx)?;

    keyboard_hook.start()?;

    Ok(Self {
      event_rx,
      _keyboard_hook: Some(keyboard_hook),
    })
  }

  /// Returns the next keybinding event from the listener.
  ///
  /// This method will block until a keybinding event is available.
  pub async fn next_event(&mut self) -> Option<KeybindingEvent> {
    self.event_rx.recv().await
  }
}
