use crate::Dispatcher;

pub enum MouseButton {
  Left,
  Right,
  Middle,
}

/// A listener for system-wide mouse events.
pub struct MouseListener {
  _dispatcher: Dispatcher,
}

impl MouseListener {
  /// Creates a new mouse listener.
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

  /// Enables or disables the mouse listener.
  pub fn enable(&mut self, enabled: bool) {
    // TODO: Implement platform-specific mouse listener enabling
  }
}
