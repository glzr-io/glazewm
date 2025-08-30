use crate::Dispatcher;

/// Listener for mouse events across the system.
///
/// The mouse listener provides events for mouse movement, clicks, and
/// other mouse interactions that occur anywhere on the system.
pub struct MouseListener {
  _dispatcher: Dispatcher,
}

impl MouseListener {
  /// Creates a new mouse listener using the provided dispatcher.
  ///
  /// The listener will use the dispatcher to receive mouse events from
  /// the platform event loop.
  pub fn new(dispatcher: Dispatcher) -> crate::Result<Self> {
    // TODO: Implement platform-specific mouse listener setup
    Ok(Self {
      _dispatcher: dispatcher,
    })
  }

  /// Returns the next mouse event from the listener.
  ///
  /// This method will block until a mouse event is available.
  pub async fn next_event(
    &mut self,
  ) -> Option<crate::platform_event::MouseMoveEvent> {
    // TODO: Implement mouse event reception
    None
  }
}
