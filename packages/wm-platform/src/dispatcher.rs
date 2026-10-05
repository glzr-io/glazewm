use std::{
  path::Path,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  thread::ThreadId,
};

use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::POINT,
    System::Environment::ExpandEnvironmentStringsW,
    UI::{
      Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON,
      },
      Shell::{
        ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS,
        SHELLEXECUTEINFOW,
      },
      WindowsAndMessaging::{
        GetCursorPos, MessageBoxW, SetCursorPos, SystemParametersInfoW,
        ANIMATIONINFO, IDYES, MB_ICONERROR, MB_ICONINFORMATION,
        MB_ICONQUESTION, MB_OK, MB_SYSTEMMODAL, MB_YESNO, SPIF_SENDCHANGE,
        SPIF_UPDATEINIFILE, SPI_GETANIMATION, SPI_SETANIMATION, SW_HIDE,
        SW_NORMAL, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
      },
    },
  },
};

use crate::{
  platform_impl, Display, DisplayDevice, MouseButton, NativeWindow, Point,
};

/// Type alias for a closure to be executed by the event loop.
pub type DispatchFn = dyn FnOnce() + Send + 'static;

/// A callback that pre-processes window procedure messages received by the
/// event loop.
///
/// Mirrors the Win32 [`WNDPROC`] signature. Returns `Some(lresult)` if
/// the message was handled, or `None` to pass it along.
///
/// [`WNDPROC`]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nc-winuser-wndproc
pub type WndProcCallback =
  dyn Fn(isize, u32, usize, isize) -> Option<isize> + Send + 'static;

/// Windows-specific extensions for `Dispatcher`.
pub trait DispatcherExtWindows {
  /// Returns the handle of the event loop's message window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn message_window_handle(&self) -> isize;

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

  /// Gets whether system-wide window transition animations are enabled.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn window_animations_enabled(&self) -> crate::Result<bool>;

  /// Enables or disables system-wide window transition animations.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_window_animations_enabled(
    &self,
    enable: bool,
  ) -> crate::Result<()>;

  /// Expands `%VAR%` environment variable references in `input`.
  ///
  /// Returns the expanded string.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  ///
  /// TODO: Remove this. Handle environment variable expansion in a
  /// unified, cross-platform way.
  fn expand_env_strings(&self, input: &str) -> crate::Result<String>;

  /// Runs the specified program using `ShellExecuteExW`.
  ///
  /// If `hide_window` is `true`, the spawned process window is hidden.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  ///
  /// TODO: Remove this. Use `shell_util::Shell::spawn` instead.
  fn shell_execute_ex(
    &self,
    program: &str,
    args: &str,
    directory: &Path,
    hide_window: bool,
  ) -> crate::Result<()>;
}

impl DispatcherExtWindows for Dispatcher {
  fn message_window_handle(&self) -> isize {
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

  fn window_animations_enabled(&self) -> crate::Result<bool> {
    let mut animation_info = ANIMATIONINFO {
      #[allow(clippy::cast_possible_truncation)]
      cbSize: std::mem::size_of::<ANIMATIONINFO>() as u32,
      iMinAnimate: 0,
    };

    unsafe {
      SystemParametersInfoW(
        SPI_GETANIMATION,
        animation_info.cbSize,
        Some(std::ptr::from_mut(&mut animation_info).cast()),
        SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
      )
    }?;

    Ok(animation_info.iMinAnimate != 0)
  }

  fn set_window_animations_enabled(
    &self,
    enable: bool,
  ) -> crate::Result<()> {
    let mut animation_info = ANIMATIONINFO {
      #[allow(clippy::cast_possible_truncation)]
      cbSize: std::mem::size_of::<ANIMATIONINFO>() as u32,
      iMinAnimate: i32::from(enable),
    };

    unsafe {
      SystemParametersInfoW(
        SPI_SETANIMATION,
        animation_info.cbSize,
        Some(std::ptr::from_mut(&mut animation_info).cast()),
        SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
      )
    }?;

    Ok(())
  }

  fn expand_env_strings(&self, input: &str) -> crate::Result<String> {
    let wide_input =
      input.encode_utf16().chain(Some(0)).collect::<Vec<_>>();

    let size = unsafe {
      ExpandEnvironmentStringsW(PCWSTR(wide_input.as_ptr()), None)
    };

    if size == 0 {
      return Err(crate::Error::Platform(format!(
        "Failed to expand environment strings in '{input}'.",
      )));
    }

    let mut buffer = vec![0u16; size as usize];
    let size = unsafe {
      ExpandEnvironmentStringsW(
        PCWSTR(wide_input.as_ptr()),
        Some(&mut buffer),
      )
    };

    // The size includes the null terminator, so subtract one.
    Ok(String::from_utf16_lossy(&buffer[..(size - 1) as usize]))
  }

  fn shell_execute_ex(
    &self,
    program: &str,
    args: &str,
    directory: &Path,
    hide_window: bool,
  ) -> crate::Result<()> {
    let program_wide =
      program.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let args_wide = args.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let directory_wide = directory
      .to_string_lossy()
      .encode_utf16()
      .chain(Some(0))
      .collect::<Vec<_>>();

    let mut exec_info = SHELLEXECUTEINFOW {
      #[allow(clippy::cast_possible_truncation)]
      cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
      lpFile: PCWSTR(program_wide.as_ptr()),
      lpParameters: PCWSTR(args_wide.as_ptr()),
      lpDirectory: PCWSTR(directory_wide.as_ptr()),
      nShow: if hide_window { SW_HIDE } else { SW_NORMAL }.0 as _,
      fMask: SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC,
      ..Default::default()
    };

    unsafe { ShellExecuteExW(&raw mut exec_info) }
      .map_err(crate::Error::from)
  }
}

/// A thread-safe dispatcher for window management operations.
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
  // `EventLoopInstaller` (to be added).
  pub(crate) fn new(
    source: Option<platform_impl::EventLoopSource>,
    stopped: Arc<AtomicBool>,
  ) -> Self {
    Self { source, stopped }
  }

  /// Stops the event loop gracefully from any thread.
  ///
  /// After calling this method, all subsequent calls to `dispatch_async()`
  /// and `dispatch_sync()` will return `Error::EventLoopStopped`.
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
  /// executed directly (synchronously).
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
      // Uses `PostMessageW` to send callbacks via window messages.
      source.send_dispatch_async(dispatch_fn)?;
    }

    Ok(())
  }

  /// Synchronously executes a closure on the event loop thread.
  ///
  /// If the current thread is the event loop thread, the function is
  /// executed directly.
  ///
  /// Returns a `Result` with the closure's return value.
  #[allow(clippy::missing_panics_doc)]
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
      .recv_timeout(std::time::Duration::from_secs(5))
      .map_err(crate::Error::ChannelRecv)
  }

  /// Gets the thread ID of the event loop thread.
  #[allow(clippy::missing_panics_doc)]
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
  /// NOTE: Does not guarantee a specific, consistent order.
  ///
  /// Returns all displays that are currently active and available for use.
  pub fn displays(&self) -> crate::Result<Vec<Display>> {
    platform_impl::all_displays(self)
  }

  /// Gets all active displays sorted left-to-right, top-to-bottom.
  ///
  /// Returns all displays that are currently active and available for
  /// use, sorted by their X coordinate (left edge), with ties broken
  /// by Y coordinate (top edge).
  ///
  /// TODO: Remove this. Instead, call `sort_monitors` after populating WM
  /// state. Need to assign workspaces after sorting monitors because of
  /// `bind_to_monitor`.
  pub fn sorted_displays(&self) -> crate::Result<Vec<Display>> {
    let displays = platform_impl::all_displays(self)?;

    let mut displays_with_bounds = displays
      .into_iter()
      .map(|display| {
        let bounds = display.bounds()?;
        crate::Result::Ok((display, bounds))
      })
      .try_collect::<Vec<_>>()?;

    displays_with_bounds.sort_by(|(_, bounds_a), (_, bounds_b)| {
      if bounds_a.x() == bounds_b.x() {
        bounds_a.y().cmp(&bounds_b.y())
      } else {
        bounds_a.x().cmp(&bounds_b.x())
      }
    });

    Ok(
      displays_with_bounds
        .into_iter()
        .map(|(display, _)| display)
        .collect(),
    )
  }

  /// Gets all display devices.
  ///
  /// NOTE: Does not guarantee a specific, consistent order.
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
    point: &Point,
  ) -> crate::Result<Display> {
    platform_impl::display_from_point(point, self)
  }

  /// Gets the primary display.
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
  /// NOTE: Does not guarantee a specific, consistent order.
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

  /// Gets the current cursor position.
  pub fn cursor_position(&self) -> crate::Result<Point> {
    let mut point = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&raw mut point) }?;

    Ok(Point {
      x: point.x,
      y: point.y,
    })
  }

  /// Gets whether the given mouse button is currently pressed.
  #[must_use]
  pub fn is_mouse_down(&self, button: &MouseButton) -> bool {
    // Virtual-key codes for mouse buttons.
    let vk_code = match button {
      MouseButton::Left => VK_LBUTTON.0,
      MouseButton::Right => VK_RBUTTON.0,
    };

    // High-order bit set indicates the key is currently down.
    let state = unsafe { GetAsyncKeyState(vk_code.into()) };
    (state.cast_unsigned() & 0x8000u16) != 0
  }

  /// Gets the top-level window at the specified point.
  pub fn window_from_point(
    &self,
    point: &Point,
  ) -> crate::Result<Option<crate::NativeWindow>> {
    platform_impl::window_from_point(point, self)
  }

  /// Sets the cursor position to the specified coordinates.
  pub fn set_cursor_position(&self, point: &Point) -> crate::Result<()> {
    unsafe { SetCursorPos(point.x, point.y) }?;

    Ok(())
  }

  /// Removes focus from the current window and focuses the desktop.
  pub fn reset_focus(&self) -> crate::Result<()> {
    platform_impl::reset_focus(self)
  }

  /// Opens File Explorer at the given path.
  pub fn open_file_explorer(&self, path: &Path) -> crate::Result<()> {
    let normalized_path = std::fs::canonicalize(path)?;
    std::process::Command::new("explorer")
      .arg(normalized_path)
      .spawn()?;

    Ok(())
  }

  /// Shows a modal error dialog with the given title and message.
  ///
  /// Blocks the current thread until the user dismisses the dialog.
  pub fn show_error_dialog(&self, title: &str, message: &str) {
    self.show_dialog(title, message, DialogKind::Error);
  }

  /// Shows a modal informational dialog with the given title and message.
  ///
  /// Blocks the current thread until the user dismisses the dialog.
  pub fn show_info_dialog(&self, title: &str, message: &str) {
    self.show_dialog(title, message, DialogKind::Info);
  }

  /// Shows a modal confirmation dialog with "Yes" and "No" buttons.
  ///
  /// Blocks the current thread until the user answers the dialog.
  ///
  /// Returns `true` if the user confirmed, otherwise `false`.
  #[must_use]
  pub fn show_confirm_dialog(&self, title: &str, message: &str) -> bool {
    self.show_dialog(title, message, DialogKind::Confirm)
  }

  /// Shows a modal dialog of the given kind.
  ///
  /// Returns `true` only if the user confirmed a [`DialogKind::Confirm`]
  /// dialog.
  ///
  /// Uses a system-modal `MessageBoxW`, which can be shown from any
  /// thread.
  // LINT: `show_dialog` is kept as a method for API consistency with the
  // rest of the dispatcher.
  #[allow(clippy::unused_self)]
  fn show_dialog(
    &self,
    title: &str,
    message: &str,
    kind: DialogKind,
  ) -> bool {
    let title_wide =
      title.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let message_wide =
      message.encode_utf16().chain(Some(0)).collect::<Vec<_>>();

    let style = MB_SYSTEMMODAL
      | match kind {
        DialogKind::Error => MB_ICONERROR | MB_OK,
        DialogKind::Info => MB_ICONINFORMATION | MB_OK,
        DialogKind::Confirm => MB_ICONQUESTION | MB_YESNO,
      };

    // SAFETY: Both strings are null-terminated and outlive the call,
    // which blocks until the dialog is dismissed.
    let result = unsafe {
      MessageBoxW(
        None,
        PCWSTR(message_wide.as_ptr()),
        PCWSTR(title_wide.as_ptr()),
        style,
      )
    };

    result == IDYES
  }
}

/// Kind of modal dialog to show to the user.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DialogKind {
  /// Dialog with an error icon and a single "OK" button.
  Error,
  /// Dialog with an info icon and a single "OK" button.
  Info,
  /// Dialog with a question icon and "Yes" / "No" buttons.
  Confirm,
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
