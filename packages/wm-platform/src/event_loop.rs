use crate::{platform_impl, Dispatcher};

/// A basic cross-platform event loop that allows for remote dispatching
/// via [`Dispatcher`].
///
/// Does not start pumping events until [`EventLoop::run`] is called.
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
  use std::time::Duration;

  use super::*;

  // TODO: Stopping the event loop isn't working here and prevents other
  // tests from running.
  #[test]
  fn event_loop_start_stop() {
    // let (event_loop, dispatcher) =
    //   EventLoop::new().expect("Failed to create event loop.");

    // // Stop the event loop after a short delay.
    // let handle = std::thread::spawn(move || {
    //   std::thread::sleep(Duration::from_millis(10));
    //   dispatcher.stop_event_loop()
    // });

    // event_loop.run().expect("Failed to run event loop.");

    // // Ensure the event loop is stopped.
    // assert!(handle.join().is_ok());
  }
}
