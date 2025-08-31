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
  use std::{
    sync::{Arc, Mutex},
    time::Duration,
  };

  use super::*;

  #[test]
  fn event_loop_start_stop_with_dispatch() {
    let (event_loop, dispatcher) =
      EventLoop::new().expect("Failed to create event loop");

    // Test that we can dispatch work and stop the event loop
    let test_value = Arc::new(Mutex::new(0));
    let test_value_clone = test_value.clone();

    let dispatcher_clone = dispatcher.clone();
    std::thread::spawn(move || {
      std::thread::sleep(Duration::from_millis(100));

      // Dispatch some work
      dispatcher_clone
        .dispatch(move || {
          *test_value_clone.lock().unwrap() = 42;
        })
        .expect("Failed to dispatch");

      // Stop the event loop after a short delay
      std::thread::sleep(Duration::from_millis(100));
      dispatcher_clone
        .stop_event_loop()
        .expect("Failed to stop event loop");
    });

    // Run the event loop (this blocks until stopped)
    event_loop.run().expect("Event loop failed");

    // Verify the dispatch actually executed
    assert_eq!(
      *test_value.lock().unwrap(),
      42,
      "Dispatched work should have executed"
    );
  }

  #[test]
  fn dispatcher_sync_dispatch() {
    let (event_loop, dispatcher) =
      EventLoop::new().expect("Failed to create event loop");

    let dispatcher_clone = dispatcher.clone();
    std::thread::spawn(move || {
      std::thread::sleep(Duration::from_millis(100));

      // Test synchronous dispatch
      let result = dispatcher_clone
        .dispatch_sync(|| 42)
        .expect("Failed to dispatch_sync");

      assert_eq!(result, 42, "dispatch_sync should return correct value");

      dispatcher_clone
        .stop_event_loop()
        .expect("Failed to stop event loop");
    });

    event_loop.run().expect("Event loop failed");
  }
}
