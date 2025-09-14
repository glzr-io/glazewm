use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc,
};

#[cfg(target_os = "macos")]
use crate::platform_impl::Application;
use crate::{platform_impl, Display, DisplayDevice, NativeWindow, Point};

/// Type alias for a closure to be executed by the event loop.
pub type DispatchFn = dyn FnOnce() + Send + 'static;

/// macOS-specific extensions for `Dispatcher`.
#[cfg(target_os = "macos")]
pub trait DispatcherExtMacOs {
  /// Gets all running applications.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn all_applications(&self) -> crate::Result<Vec<Application>>;
}

#[cfg(target_os = "macos")]
impl DispatcherExtMacOs for Dispatcher {
  fn all_applications(&self) -> crate::Result<Vec<Application>> {
    platform_impl::all_applications(self)
  }
}

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
/// ```rust,no_run
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
///   }).unwrap();
/// });
///
/// event_loop.run()
/// # }
/// ```
#[derive(Clone)]
pub struct Dispatcher {
  source: Option<platform_impl::EventLoopSource>,
  stopped: Arc<AtomicBool>,
}

impl Dispatcher {
  // TODO: Allow for source to be resolved after creation when used via
  // `EventLoopInstaller`.
  pub(crate) fn new(
    source: Option<platform_impl::EventLoopSource>,
    stopped: Arc<AtomicBool>,
  ) -> Self {
    Self { source, stopped }
  }

  /// Stops the event loop gracefully from any thread.
  ///
  /// After calling this method, all subsequent `dispatch()` and
  /// `dispatch_sync()` calls will return `Error::EventLoopStopped`.
  pub fn stop_event_loop(&self) -> crate::Result<()> {
    // Set stopped flag to prevent new dispatches.
    self.stopped.store(true, Ordering::SeqCst);

    // Signal platform-specific event loop to stop.
    if let Some(source) = &self.source {
      source.send_stop()?;
    }

    Ok(())
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
    // Check if stopped first.
    if self.stopped.load(Ordering::SeqCst) {
      return Err(crate::Error::EventLoopStopped);
    }

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
    let (result_tx, result_rx) = std::sync::mpsc::channel();

    self.dispatch(move || {
      let result = dispatch_fn();

      if result_tx.send(result).is_err() {
        tracing::error!("Failed to send closure result.");
      }
    })?;

    result_rx.recv().map_err(crate::Error::ChannelRecv)
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

  /// Gets all visible windows from all running applications.
  ///
  /// Returns a vector of `NativeWindow` instances for windows that are
  /// currently visible (not minimized or hidden).
  pub fn visible_windows(&self) -> crate::Result<Vec<NativeWindow>> {
    platform_impl::visible_windows(self)
  }

  pub fn nearest_monitor(
    &self,
    _native_window: &NativeWindow,
  ) -> crate::Result<Display> {
    // TODO: Implement this.
    platform_impl::primary_display(self)
  }

  pub fn mouse_position(&self) -> crate::Result<Point> {
    todo!()
  }
}

impl std::fmt::Debug for Dispatcher {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "EventLoopDispatcher")
  }
}

#[cfg(test)]
mod tests {
  use std::{
    sync::{Arc, Mutex},
    time::Duration,
  };

  use crate::EventLoop;

  #[test]
  fn dispatch_after_stop_fails() {
    let (_event_loop, dispatcher) = EventLoop::new().unwrap();

    dispatcher
      .stop_event_loop()
      .expect("Failed to stop dispatcher.");

    // Try to dispatch asynchronously - should fail.
    let result = dispatcher.dispatch(|| {});
    assert!(matches!(result, Err(crate::Error::EventLoopStopped)));

    // Try dispatch synchronously - should fail.
    let sync_result: crate::Result<i32> = dispatcher.dispatch_sync(|| 69);
    assert!(matches!(sync_result, Err(crate::Error::EventLoopStopped)));
  }

  // #[test]
  // fn multiple_dispatches_execute_in_order() {
  //   const ITERATIONS: usize = 500;

  //   let (event_loop, dispatcher) = EventLoop::new().unwrap();

  //   let order = Arc::new(Mutex::new(Vec::new()));
  //   let order_clone = order.clone();

  //   std::thread::spawn(move || {
  //     for index in 1..=ITERATIONS {
  //       let order_clone = order_clone.clone();
  //       dispatcher
  //         .dispatch(move || {
  //           order_clone.lock().unwrap().push(index);
  //         })
  //         .unwrap();
  //     }

  //     dispatcher.stop_event_loop().unwrap();
  //   });

  //   event_loop.run().unwrap();
  //   assert_eq!(
  //     *order.lock().unwrap(),
  //     (1..=ITERATIONS).collect::<Vec<_>>()
  //   );
  // }

  #[test]
  fn dispatch_from_different_threads() {
    let (event_loop, dispatcher) = EventLoop::new().unwrap();
    let results = Arc::new(Mutex::new(Vec::new()));

    let thread_handles: Vec<_> = (0..3)
      .map(|index| {
        let dispatcher = dispatcher.clone();
        let results = results.clone();
        std::thread::spawn(move || {
          dispatcher
            .dispatch(move || {
              results.lock().unwrap().push(index);
            })
            .unwrap();
        })
      })
      .collect();

    std::thread::spawn(move || {
      std::thread::sleep(Duration::from_millis(100));
      dispatcher.stop_event_loop().unwrap();
    });

    event_loop.run().unwrap();
    for handle in thread_handles {
      handle.join().unwrap();
    }

    let mut results = results.lock().unwrap().clone();
    results.sort_unstable();
    assert_eq!(results, vec![0, 1, 2]);
  }
}
