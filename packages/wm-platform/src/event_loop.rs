use crate::{platform_impl, Dispatcher};

/// A basic cross-platform event loop that allows for remote dispatching
/// via [`Dispatcher`].
///
/// # Platform-specific
///
/// - **macOS**: Must be called from the main thread. Runs
///   `CFRunLoopRun()`.
/// - **Windows**: Can be called from any thread. Runs a Win32 message
///   loop.
pub struct EventLoop {
  inner: platform_impl::EventLoop,
}

impl EventLoop {
  /// Creates a new event loop and dispatcher.
  pub fn new() -> crate::Result<(Self, Dispatcher)> {
    let (event_loop, dispatcher) = platform_impl::EventLoop::new()?;
    Ok((Self { inner: event_loop }, dispatcher))
  }

  /// Runs the event loop, blocking the current thread until shutdown.
  ///
  /// # Platform-specific
  ///
  /// - **macOS**: Must be called from the main thread. Runs
  ///   `CFRunLoopRun()`.
  /// - **Windows**: Can be called from any thread. Runs Win32 message
  ///   loop.
  pub fn run(self) -> crate::Result<()> {
    self.inner.run()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_event_loop_struct_exists() {
    let event_loop = EventLoop::new();
    assert!(event_loop.is_ok());

    if let Ok((event_loop, _)) = event_loop {
      // TODO: Without the `event_loop.run()` call, macOS crashes. But this
      // then causes the test case to run forever.
      event_loop.run();
    }
  }
}
