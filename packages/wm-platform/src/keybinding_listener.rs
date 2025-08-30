use crate::Dispatcher;

/// Listener for keybinding events across the system.
///
/// The keybinding listener provides events for registered keyboard
/// shortcuts and hotkeys that occur anywhere on the system.
pub struct KeybindingListener {
  _dispatcher: Dispatcher,
}

impl KeybindingListener {
  /// Creates a new keybinding listener using the provided dispatcher.
  ///
  /// The listener will use the dispatcher to receive keybinding events
  /// from the platform event loop.
  pub fn new(dispatcher: Dispatcher) -> crate::Result<Self> {
    // TODO: Implement platform-specific keybinding listener setup
    Ok(Self {
      _dispatcher: dispatcher,
    })
  }

  /// Returns the next keybinding event from the listener.
  ///
  /// This method will block until a keybinding event is available.
  pub async fn next_event(
    &mut self,
  ) -> Option<crate::platform_event::KeybindingEvent> {
    // TODO: Implement keybinding event reception
    None
  }
}
