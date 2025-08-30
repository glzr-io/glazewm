use std::{
  cell::RefCell,
  sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock,
  },
};

use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostMessageW, RegisterWindowMessageW},
  },
};

thread_local! {
  /// Custom message ID for dispatching callbacks to the event loop thread.
  ///
  /// `WPARAM` contains a `Box<Box<dyn FnOnce()>>` that must be retrieved
  /// with `Box::from_raw`, and `LPARAM` is unused.
  ///
  /// This message is sent using `PostMessageW` and handled in
  /// [`EventLoop::window_proc`].
  static WM_DISPATCH_CALLBACK: u32 = RegisterWindowMessageW(w!("GlazeWM:Dispatch"));
}

/// Type alias for the callback function used with dispatches.
type DispatchFn = Box<Box<dyn FnOnce() + Send + 'static>>;

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
  pub fn new(
    message_window_handle: crate::WindowHandle,
    thread_id: u32,
  ) -> Self {
    Self {
      message_window_handle,
      thread_id,
    }
  }

  /// Dispatches a closure to be executed on the event loop thread.
  ///
  /// This is a fire-and-forget operation that schedules the closure to
  /// run asynchronously. The calling thread does not wait for the closure
  /// to complete and no result is returned.
  ///
  /// Returns `Ok(())` if the closure was successfully queued.
  pub fn dispatch<F>(&self, dispatch_fn: F) -> crate::Result<()>
  where
    F: FnOnce() + Send + 'static,
  {
    use windows::Win32::UI::WindowsAndMessaging::{
      PostMessageW, HWND, LPARAM, WPARAM,
    };

    // Double box the callback to avoid `STATUS_ACCESS_VIOLATION`.
    // Ref Tao's implementation: https://github.com/tauri-apps/tao/blob/dev/src/platform_impl/windows/event_loop.rs#L596
    let boxed_callback = Box::new(dispatch_fn);
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
        Err(crate::Error::WindowMessage(
          "Failed to post message".to_string(),
        ))
      }
    }
  }

  /// Dispatches a closure to be executed on the event loop thread and
  /// blocks until it completes, returning its result.
  ///
  /// This method synchronously executes the closure, blocking the calling
  /// thread until the closure finishes executing.
  ///
  /// Returns a result containing the closure's return value if successful.
  pub fn dispatch_sync<F, R>(&self, dispatch_fn: F) -> crate::Result<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    use windows::Win32::System::Threading::GetCurrentThreadId;

    // Execute the function directly if we're already on the event loop
    // thread.
    if unsafe { GetCurrentThreadId() } == self.thread_id {
      return Ok(dispatch_fn());
    }

    let (res_tx, res_rx) = std::sync::mpsc::channel();

    self.dispatch(move || {
      let res = dispatch_fn();

      if res_tx.send(res).is_err() {
        tracing::error!("Failed to send closure result.");
      }
    })?;

    res_rx.recv().map_err(crate::Error::ChannelRecv)
  }

  // TODO: Remove `name` arg after testing - keeping for backward
  // compatibility
  pub fn run<F>(&self, name: &str, callback: F) -> anyhow::Result<()>
  where
    F: FnOnce() + Send + 'static,
  {
    tracing::debug!("Dispatching callback: {name}.");
    self
      .dispatch(callback)
      .map_err(|e| anyhow::anyhow!("{}", e))
  }
}
