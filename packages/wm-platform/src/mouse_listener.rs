use crate::{platform_impl, Dispatcher};

pub enum MouseButton {
  Left,
  Right,
}

/// A listener for system-wide mouse events.
pub struct MouseListener {
  inner: platform_impl::MouseListener,
}

impl MouseListener {
  /// Creates a new mouse listener.
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    Ok(Self {
      inner: platform_impl::MouseListener::new(dispatcher)?,
    })
  }

  /// Returns the next mouse event from the listener.
  ///
  /// This will block until a mouse event is available.
  pub async fn next_event(
    &mut self,
  ) -> Option<crate::platform_event::MouseEvent> {
    self.inner.event_rx.recv().await
  }

  /// Enables or disables the mouse listener.
  pub fn enable(&mut self, enabled: bool) {
    self.inner.enable(enabled);
  }
}
