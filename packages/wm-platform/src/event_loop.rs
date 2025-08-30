use crate::{platform_impl, Dispatcher};

/// A platform-specific event loop for handling window management events.
///
/// The event loop must be run to process platform events. Once created,
/// it provides a dispatcher that can be used to query system state and
/// create listeners.
///
/// # Platform-specific
///
/// - **macOS**: Must be called from the main thread. Runs
///   `CFRunLoopRun()`.
/// - **Windows**: Can be called from any thread. Runs Win32 message loop.
pub struct EventLoop {
  inner: platform_impl::EventLoop,
}

impl EventLoop {
  /// Creates a new event loop and dispatcher.
  ///
  /// The dispatcher can be used immediately to create listeners and query
  /// system state, but events will only be received after calling `run()`.
  pub fn new() -> crate::Result<(Self, Dispatcher)> {
    let (inner_loop, inner_dispatcher) = platform_impl::EventLoop::new()?;
    let dispatcher = Dispatcher::new(inner_dispatcher);

    Ok((Self { inner: inner_loop }, dispatcher))
  }

  /// Runs the event loop, blocking until shutdown.
  ///
  /// This method will block the current thread and process platform
  /// events. The event loop continues running until shutdown is
  /// requested.
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
