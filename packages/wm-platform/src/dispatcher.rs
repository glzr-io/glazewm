use std::{
  path::Path,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  thread::ThreadId,
};

#[cfg(target_os = "macos")]
use crate::platform_impl::Application;
use crate::{
  platform_impl, Display, DisplayDevice, MouseButton, NativeWindow, Point,
};

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

  /// Checks whether accessibility permissions are granted.
  ///
  /// If `prompt` is `true`, a dialog will be shown to the user to request
  /// accessibility permissions.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn has_ax_permission(&self, prompt: bool) -> bool;
}

#[cfg(target_os = "macos")]
impl DispatcherExtMacOs for Dispatcher {
  fn all_applications(&self) -> crate::Result<Vec<Application>> {
    platform_impl::all_applications(self)
  }

  fn has_ax_permission(&self, prompt: bool) -> bool {
    use objc2_application_services::{
      kAXTrustedCheckOptionPrompt, AXIsProcessTrustedWithOptions,
    };
    use objc2_core_foundation::{CFBoolean, CFDictionary};

    let options = CFDictionary::from_slices(
      &[unsafe { kAXTrustedCheckOptionPrompt }],
      &[CFBoolean::new(prompt)],
    );

    unsafe { AXIsProcessTrustedWithOptions(Some(options.as_ref())) }
  }
}

/// Windows-specific extensions for `Dispatcher`.
#[cfg(target_os = "windows")]
pub trait DispatcherExtWindows {
  /// Returns the handle of the event loop's message window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn message_window_handle(&self) -> crate::WindowHandle;

  /// Registers a callback to pre-process messages in the event loop's
  /// window procedure.
  ///
  /// Returns a unique ID that can be passed to
  /// `deregister_wndproc_callback` to remove the callback.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn register_wndproc_callback(
    &self,
    callback: Box<crate::WndProcCallback>,
  ) -> crate::Result<usize>;

  /// Removes a previously registered window procedure callback by its ID.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn deregister_wndproc_callback(&self, id: usize) -> crate::Result<()>;
}

#[cfg(target_os = "windows")]
impl DispatcherExtWindows for Dispatcher {
  fn message_window_handle(&self) -> crate::WindowHandle {
    self.source.as_ref().unwrap().message_window_handle
  }

  fn register_wndproc_callback(
    &self,
    callback: Box<crate::WndProcCallback>,
  ) -> crate::Result<usize> {
    self
      .source
      .as_ref()
      .unwrap()
      .register_wndproc_callback(callback)
  }

  fn deregister_wndproc_callback(&self, id: usize) -> crate::Result<()> {
    self
      .source
      .as_ref()
      .unwrap()
      .deregister_wndproc_callback(id)
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
///   dispatcher.dispatch_async(|| {
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
  pub fn dispatch_async<F>(&self, dispatch_fn: F) -> crate::Result<()>
  where
    F: FnOnce() + Send + 'static,
  {
    // Check if stopped first.
    if self.stopped.load(Ordering::SeqCst) {
      return Err(crate::Error::EventLoopStopped);
    }

    // Execute the function directly if already on the event loop thread.
    if self.is_event_loop_thread() {
      dispatch_fn();
      return Ok(());
    }

    if let Some(source) = &self.source {
      // Platform-specific behavior:
      // * On Windows, this uses `PostMessageW` to send callbacks via
      //   window messages.
      // * On macOS, this uses `CFRunLoopSourceSignal` to wake the run loop
      //   and process callbacks.
      source.send_dispatch_async(dispatch_fn)?;
    }

    Ok(())
  }

  /// Synchronously executes a closure on the event loop thread.
  ///
  /// If the current thread is the event loop thread, the function is
  /// executed directly. Otherwise, this method synchronously dispatches
  /// and executes the closure.
  ///
  /// Returns a result with the closure's return value.
  pub fn dispatch_sync<F, R>(&self, dispatch_fn: F) -> crate::Result<R>
  where
    F: FnOnce() -> R + Send,
    R: Send,
  {
    // Check if stopped first.
    if self.stopped.load(Ordering::SeqCst) {
      return Err(crate::Error::EventLoopStopped);
    }

    // Execute the function directly if already on the event loop thread.
    if self.is_event_loop_thread() {
      return Ok(dispatch_fn());
    }

    let (result_tx, result_rx) = std::sync::mpsc::channel();

    // TODO: Block until event loop source is set.
    self.source.as_ref().unwrap().send_dispatch_sync(move || {
      let result = dispatch_fn();

      if result_tx.send(result).is_err() {
        tracing::error!("Failed to send closure result.");
      }
    })?;

    result_rx
      .recv_timeout(std::time::Duration::from_millis(3000))
      .map_err(crate::Error::ChannelRecv)
  }

  /// Gets the thread ID of the event loop thread.
  #[must_use]
  pub fn thread_id(&self) -> ThreadId {
    // TODO: Block until event loop source is set.
    self.source.as_ref().unwrap().thread_id
  }

  /// Gets whether the current thread is the event loop thread.
  #[must_use]
  fn is_event_loop_thread(&self) -> bool {
    std::thread::current().id() == self.thread_id()
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
  pub fn display_devices(&self) -> crate::Result<Vec<DisplayDevice>> {
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

  /// Gets the nearest display to a window.
  ///
  /// Returns the display that contains the largest area of the window's
  /// frame. Defaults to the primary display if no overlap is found.
  pub fn nearest_display(
    &self,
    native_window: &NativeWindow,
  ) -> crate::Result<Display> {
    platform_impl::nearest_display(native_window, self)
  }

  /// Gets all visible windows from all running applications.
  ///
  /// Returns a vector of `NativeWindow` instances for windows that are
  /// not hidden and on the current virtual desktop.
  pub fn visible_windows(&self) -> crate::Result<Vec<NativeWindow>> {
    platform_impl::visible_windows(self)
  }

  /// Gets the currently focused (foreground) window.
  ///
  /// This may be the desktop window if no window has focus.
  pub fn focused_window(&self) -> crate::Result<NativeWindow> {
    platform_impl::focused_window(self)
  }

  pub fn cursor_position(&self) -> crate::Result<Point> {
    #[cfg(target_os = "macos")]
    {
      use objc2_core_graphics::CGEvent;

      let event = unsafe { CGEvent::new(None) };
      let point = unsafe { CGEvent::location(event.as_deref()) };

      #[allow(clippy::cast_possible_truncation)]
      Ok(Point {
        x: point.x as i32,
        y: point.y as i32,
      })
    }
    #[cfg(target_os = "windows")]
    {
      use windows::Win32::{
        Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos,
      };

      let mut point = POINT { x: 0, y: 0 };
      unsafe { GetCursorPos(&raw mut point) }?;

      Ok(Point {
        x: point.x,
        y: point.y,
      })
    }
  }

  #[must_use]
  pub fn is_mouse_down(&self, button: &MouseButton) -> bool {
    #[cfg(target_os = "macos")]
    {
      use objc2_app_kit::NSEvent;

      let bit_index = match button {
        MouseButton::Left => 0usize,
        MouseButton::Right => 1usize,
      };

      // Check if bit at corresponding index is set in the bitmask.
      let pressed_mask = unsafe { NSEvent::pressedMouseButtons() };
      (pressed_mask & (1usize << bit_index)) != 0
    }
    #[cfg(target_os = "windows")]
    {
      use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON,
      };

      // Virtual-key codes for mouse buttons.
      let vk_code = match button {
        MouseButton::Left => VK_LBUTTON.0,
        MouseButton::Right => VK_RBUTTON.0,
      };

      // High-order bit set indicates the key is currently down.
      let state = unsafe { GetAsyncKeyState(vk_code.into()) };
      (state as u16 & 0x8000) != 0
    }
  }

  pub fn window_from_point(
    &self,
    point: &Point,
  ) -> crate::Result<Option<crate::NativeWindow>> {
    platform_impl::window_from_point(point, self)
  }

  /// Sets the cursor position to the specified coordinates.
  pub fn set_cursor_position(&self, point: &Point) -> crate::Result<()> {
    #[cfg(target_os = "macos")]
    {
      use objc2_core_foundation::CGPoint;
      use objc2_core_graphics::{CGError, CGWarpMouseCursorPosition};

      let point = CGPoint {
        x: f64::from(point.x),
        y: f64::from(point.y),
      };

      if unsafe { CGWarpMouseCursorPosition(point) } != CGError::Success {
        return Err(crate::Error::Platform(
          "Failed to set cursor position.".to_string(),
        ));
      }
    }
    #[cfg(target_os = "windows")]
    {
      use windows::Win32::UI::WindowsAndMessaging::SetCursorPos;

      unsafe { SetCursorPos(point.x, point.y) }?;
    }

    Ok(())
  }

  /// Removes focus from the current window and focuses the desktop.
  ///
  /// # Platform-specific
  ///
  /// - **macOS**: Uses
  ///   `NSApplicationActivationOptions::ActivateAllWindows` to activate
  ///   the desktop application.
  pub fn reset_focus(&self) -> crate::Result<()> {
    platform_impl::reset_focus(self)
  }

  /// Opens the OS-specific file explorer at the specified path.
  ///
  /// # Platform-specific
  ///
  /// - **Windows**: Uses `explorer` to open the file explorer.
  /// - **macOS**: Uses `open` to open the file explorer.
  pub fn open_file_explorer(&self, path: &Path) -> crate::Result<()> {
    #[cfg(target_os = "windows")]
    {
      std::process::Command::new("explorer").arg(path).spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
      std::process::Command::new("open")
        .arg(path)
        .arg("-R")
        .spawn()?;
    }

    Ok(())
  }
}

impl std::fmt::Debug for Dispatcher {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "EventLoopDispatcher")
  }
}

#[cfg(test)]
mod tests {
  use std::sync::{Arc, Mutex};

  use crate::EventLoop;

  #[test]
  fn dispatch_after_stop_fails() {
    let (_event_loop, dispatcher) = EventLoop::new().unwrap();

    dispatcher
      .stop_event_loop()
      .expect("Failed to stop dispatcher.");

    // Try to dispatch asynchronously - should fail.
    let result = dispatcher.dispatch_sync(|| {});
    assert!(matches!(result, Err(crate::Error::EventLoopStopped)));

    // Try dispatch synchronously - should fail.
    let sync_result: crate::Result<i32> = dispatcher.dispatch_sync(|| 69);
    assert!(matches!(sync_result, Err(crate::Error::EventLoopStopped)));
  }

  #[test]
  fn dispatch_sync_executes_in_order() {
    const ITERATIONS: usize = 5000;

    let (event_loop, dispatcher) = EventLoop::new().unwrap();

    let order = Arc::new(Mutex::new(Vec::new()));
    let order_clone = order.clone();

    std::thread::spawn(move || {
      for index in 1..=ITERATIONS {
        dispatcher
          .dispatch_sync(|| {
            order_clone.lock().unwrap().push(index);
          })
          .unwrap();
      }

      dispatcher.stop_event_loop().unwrap();
    });

    event_loop.run().unwrap();
    assert_eq!(
      *order.lock().unwrap(),
      (1..=ITERATIONS).collect::<Vec<_>>()
    );
  }

  #[test]
  fn dispatch_sync_from_different_threads() {
    // Stress test with many threads calling `dispatch_sync`
    // simultaneously. Ensure that dispatching doesn't deadlock.
    const NUM_THREADS: usize = 10;
    const ITERATIONS: usize = 1000;

    let (event_loop, dispatcher) = EventLoop::new().unwrap();
    let counter = Arc::new(Mutex::new(0));

    let thread_handles: Vec<_> = (0..NUM_THREADS)
      .map(|_| {
        let counter = counter.clone();
        let dispatcher = dispatcher.clone();
        std::thread::spawn(move || {
          for _ in 0..ITERATIONS {
            dispatcher
              .dispatch_sync(|| {
                let mut count = counter.lock().unwrap();
                *count += 1;
              })
              .unwrap();
          }
        })
      })
      .collect();

    std::thread::spawn(move || {
      // Wait for all threads to finish.
      for handle in thread_handles {
        handle.join().unwrap();
      }
      dispatcher.stop_event_loop().unwrap();
    });

    event_loop.run().unwrap();

    assert_eq!(*counter.lock().unwrap(), NUM_THREADS * ITERATIONS);
  }

  #[test]
  fn dispatch_sync_with_nested() {
    // Test that calling `dispatch_sync` from within a `dispatch_sync`
    // callback works correctly (should execute directly without blocking).
    let (event_loop, dispatcher) = EventLoop::new().unwrap();
    let result = Arc::new(Mutex::new(Vec::new()));

    let result_clone = result.clone();
    std::thread::spawn(move || {
      dispatcher
        .dispatch_sync(|| {
          result_clone.lock().unwrap().push(1);

          // Nested `dispatch_sync` - should execute immediately since it's
          // already on the event loop thread.
          dispatcher
            .dispatch_sync(|| {
              result_clone.lock().unwrap().push(2);
            })
            .unwrap();

          result_clone.lock().unwrap().push(3);
        })
        .unwrap();

      dispatcher.stop_event_loop().unwrap();
    });

    event_loop.run().unwrap();
    assert_eq!(*result.lock().unwrap(), vec![1, 2, 3]);
  }
}
