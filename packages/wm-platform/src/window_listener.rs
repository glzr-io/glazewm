use crate::{platform_impl, Dispatcher, WindowEvent};

/// Listener for system-wide window events.
pub struct WindowListener {
  inner: platform_impl::WindowListener,
}

impl WindowListener {
  /// Creates a new window listener.
  pub fn new(dispatcher: Dispatcher) -> crate::Result<Self> {
    let inner =
      platform_impl::WindowListener::new(dispatcher.inner().clone())
        .map_err(|e| crate::Error::Platform(e.to_string()))?;

    Ok(Self { inner })
  }

  /// Returns the next window event from the listener.
  ///
  /// This method will block until a window event is available.
  pub async fn next_event(&mut self) -> Option<WindowEvent> {
    self.inner.next_event().await
  }
}
