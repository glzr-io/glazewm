/// Cross-platform dispatcher for sending callbacks to an event loop.
///
/// `EventLoopDispatcher` provides a thread-safe interface for dispatching
/// closures to be executed on the event loop thread. It abstracts away
/// platform differences between Windows (Win32 messages) and macOS
/// (CFRunLoop sources).
///
/// # Thread safety
///
/// This type is `Send + Sync` and can be safely cloned and shared across
/// threads.
///
/// # Platform-specific behavior
///
/// - **Windows**: Uses `PostMessageW` to send callbacks via window
///   messages.
/// - **macOS**: Uses `CFRunLoopSourceSignal` to wake the run loop and
///   process callbacks.
///
/// # Example usage
///
/// ```rust
/// use std::thread;
///
/// let (event_loop, dispatcher) = EventLoop::new()?;
///
/// // Dispatch from another thread.
/// let dispatcher_clone = dispatcher.clone();
/// thread::spawn(move || {
///     dispatcher_clone.dispatch(|| {
///         println!("Running on event loop thread!");
///     }).unwrap();
/// });
///
/// // Clean shutdown.
/// event_loop.shutdown()?;
/// ```
#[derive(Clone)]
pub struct EventLoopDispatcher {
  message_window_handle: crate::WindowHandle,
  thread_id: u32,
}

impl EventLoopDispatcher {
  /// Dispatches a callback to be executed on the Win32 message loop
  /// thread.
  ///
  /// # Arguments
  /// * `callback` - A closure that will be executed on the message loop
  ///   thread.
  // TODO: Remove `name` arg after testing
  pub fn run<F>(&self, name: &str, callback: F) -> anyhow::Result<()>
  where
    F: FnOnce() + Send + 'static,
  {
    tracing::debug!("Dispatching callback: {name}.");

    // Double box the callback to avoid `STATUS_ACCESS_VIOLATION`.
    // Ref Tao's implementation: https://github.com/tauri-apps/tao/blob/dev/src/platform_impl/windows/event_loop.rs#L596
    let boxed_callback = Box::new(callback);
    let boxed_callback2: DispatchFn = Box::new(boxed_callback);

    // Leak to a raw pointer to then be passed as `WPARAM` in the message.
    let callback_ptr = Box::into_raw(boxed_callback2);

    unsafe {
      if PostMessageW(
        HWND(self.message_window_handle),
        *WM_DISPATCH_CALLBACK,
        WPARAM(callback_ptr as _),
        LPARAM(0),
      )
      .is_ok()
      {
        Ok(())
      } else {
        // If `PostMessage` fails, we need to clean up the callback.
        let _ = Box::from_raw(callback_ptr);
        Err(anyhow::anyhow!("Failed to post message"))
      }
    }
  }
}
