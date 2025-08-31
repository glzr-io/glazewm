use wm_common::Point;

use crate::{platform_impl, Display, DisplayDevice, NativeWindow};

/// Type alias for a closure to be executed by the event loop.
pub type DispatchFn = dyn FnOnce() + Send + 'static;

/// A thread-safe dispatcher for various cross-platform operations.
///
/// On macOS, operations are automatically dispatched to the main thread
/// whenever necessary.
///
/// # Thread safety
///
/// This type is `Send + Sync` and can be cheaply cloned and shared across
/// threads.
///
/// # Example usage
///
/// ```rust
/// use wm_platform::EventLoop;
/// use std::thread;
///
/// # fn main() -> wm_platform::Result<()> {
/// let (event_loop, dispatcher) = EventLoop::new()?;
///
/// // Dispatch from another thread.
/// thread::spawn(move || {
///   dispatcher.dispatch(|| {
///     println!("This is running on the event loop thread!");
///     assert!(true);
///   }).unwrap();
/// });
///
/// event_loop.run()
/// # }
/// ```
#[derive(Clone)]
pub struct Dispatcher {
  source: Option<platform_impl::EventLoopSource>,
}

impl Dispatcher {
  // TODO: Allow for source to be resolved after creation when used via
  // `EventLoopInstaller`.
  pub(crate) fn new(
    source: Option<platform_impl::EventLoopSource>,
  ) -> Self {
    Self { source }
  }

  /// Asynchronously executes a closure on the event loop thread.
  ///
  /// If the current thread is the event loop thread, the function is
  /// executed directly. Otherwise, this is a fire-and-forget operation
  /// that schedules the closure to run asynchronously.
  ///
  /// Returns `Ok(())` if the closure was successfully queued. No result is
  /// returned.
  pub fn dispatch<F>(&self, dispatch_fn: F) -> crate::Result<()>
  where
    F: FnOnce() + Send + 'static,
  {
    // Execute the function directly if already on the main thread.
    if self.is_main_thread() {
      dispatch_fn();
      return Ok(());
    }

    if let Some(source) = &self.source {
      // Platform-specific behavior:
      // * On Windows, this uses `PostMessageW` to send callbacks via
      //   window messages.
      // * On macOS, this uses `CFRunLoopSourceSignal` to wake the run loop
      //   and process callbacks.
      source.send_dispatch(Box::new(dispatch_fn))?;
    }

    Ok(())
  }

  /// Synchronously executes a closure on the event loop thread.
  ///
  /// If the current thread is the event loop thread, the function is
  /// executed directly. Otherwise, this method synchronously executes
  /// the closure, blocking the calling thread until the closure finishes
  /// executing.
  ///
  /// Returns a result with the closure's return value.
  pub fn dispatch_sync<F, R>(&self, dispatch_fn: F) -> crate::Result<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    // Execute the function directly if already on the main thread.
    if self.is_main_thread() {
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

  /// Get whether the current thread is the main thread.
  #[must_use]
  pub fn is_main_thread(&self) -> bool {
    #[cfg(target_os = "macos")]
    {
      use objc2::MainThreadMarker;
      MainThreadMarker::new().is_some()
    }
    #[cfg(target_os = "windows")]
    {
      use windows::Win32::System::Threading::GetCurrentThreadId;
      self.source.thread_id == unsafe { GetCurrentThreadId() }
    }
  }

  /// Gets all active displays.
  ///
  /// Returns all displays that are currently active and available for use.
  pub fn displays(&self) -> crate::Result<Vec<Display>> {
    platform_impl::all_displays(self)
  }

  /// Gets all display devices.
  ///
  /// Returns all display devices including active, inactive, and
  /// disconnected ones.
  pub fn all_display_devices(&self) -> crate::Result<Vec<DisplayDevice>> {
    platform_impl::all_display_devices(self)
  }

  /// Gets the display containing the specified point.
  ///
  /// If no display contains the point, returns the primary display.
  pub fn display_from_point(
    &self,
    point: Point,
  ) -> crate::Result<Display> {
    platform_impl::display_from_point(point, self)
  }

  /// Gets the primary display.
  ///
  /// # Platform-specific
  ///
  /// - **macOS**: Returns the display containing the menu bar.
  pub fn primary_display(&self) -> crate::Result<Display> {
    platform_impl::primary_display(self)
  }

  /// Gets all windows.
  ///
  /// Returns all windows that are currently on-screen and meet the
  /// filtering criteria, excluding system windows like Dock, menu bar,
  /// and desktop elements.
  ///
  /// # Platform-specific
  ///
  /// - **macOS**: Uses `CGWindowListCopyWindowInfo` to enumerate windows
  ///   and filters out system applications and UI elements.
  pub fn all_windows(&self) -> crate::Result<Vec<NativeWindow>> {
    platform_impl::all_windows(self)
  }

  /// Gets all windows from all running applications.
  ///
  /// Returns a vector of `NativeWindow` instances for all windows
  /// from all running applications, including hidden applications.
  pub fn all_applications(&self) -> crate::Result<Vec<NativeWindow>> {
    platform_impl::all_applications(self)
  }

  /// Gets all visible windows from all running applications.
  ///
  /// Returns a vector of `NativeWindow` instances for windows that are
  /// currently visible (not minimized or hidden).
  pub fn visible_windows(&self) -> crate::Result<Vec<NativeWindow>> {
    platform_impl::visible_windows(self)
  }
}

impl std::fmt::Debug for Dispatcher {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "EventLoopDispatcher")
  }
}
