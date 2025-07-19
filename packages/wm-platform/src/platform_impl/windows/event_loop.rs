use std::{
  cell::RefCell,
  sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock,
  },
  thread::{self, JoinHandle},
};

use anyhow::bail;
use windows::{
  core::s,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
      DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
      PostMessageW, PostThreadMessageW, RegisterWindowMessageA,
      TranslateMessage, MSG, PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND,
      PBT_APMSUSPEND, WM_DEVICECHANGE, WM_DISPLAYCHANGE, WM_INPUT,
      WM_POWERBROADCAST, WM_QUIT, WM_SETTINGCHANGE,
    },
  },
};

use crate::platform_impl::{DisplayHook, MouseHook};

/// Custom message ID for our dispatch mechanism.
static WM_DISPATCH_CALLBACK: LazyLock<u32> = LazyLock::new(|| unsafe {
  RegisterWindowMessageA(s!("GlazeWM:Dispatch"))
});

/// Type alias for the callback function used with dispatches.
type DispatchFn = Box<Box<dyn FnOnce() + Send + 'static>>;

/// Represents a Win32 message loop that runs on a separate thread.
///
/// Callbacks can be remotely dispatched to the event loop thread using
/// [`EventLoop::dispatch`].
pub struct EventLoop {
  message_window_handle: crate::WindowHandle,
  thread_handle: Option<JoinHandle<anyhow::Result<()>>>,
  thread_id: u32,
}

impl EventLoop {
  /// Creates a new Win32 [`EventLoop`] and starts the message loop in a
  /// separate thread.
  pub fn new() -> anyhow::Result<Self> {
    let (sender, receiver) =
      tokio::sync::oneshot::channel::<(crate::WindowHandle, u32)>();

    let thread_handle = thread::spawn(move || -> anyhow::Result<()> {
      // Create a hidden window on the current thread.
      let window_handle =
        super::Platform::create_message_window(Some(Self::window_proc))?;

      let thread_id = unsafe { GetCurrentThreadId() };

      // Send the window handle and thread ID back to the main thread. Will
      // only fail if the receiver was closed, which would be due to the
      // main thread erroring - so just bail.
      if sender.send((window_handle, thread_id)).is_err() {
        unsafe { DestroyWindow(HWND(window_handle)) }?;
        bail!("Failed to send window handle back to main thread, channel was closed.");
      }

      // Run the message loop. This will block until `WM_QUIT` is
      // dispatched.
      Self::run_message_loop();

      tracing::info!("Event loop thread exiting.");
      unsafe { DestroyWindow(HWND(window_handle)) }?;

      Ok(())
    });

    // Wait for the window handle and thread ID.
    let (window_handle, thread_id) = receiver.blocking_recv()?;

    Ok(EventLoop {
      message_window_handle: window_handle,
      thread_handle: Some(thread_handle),
      thread_id,
    })
  }

  /// Dispatches a callback to be executed on the Win32 message loop thread
  ///
  /// # Arguments
  /// * `callback` - A closure that will be executed on the message loop
  ///   thread
  // TODO: Remove `name` arg after testing
  pub fn dispatch<F>(&self, name: &str, callback: F) -> anyhow::Result<()>
  where
    F: FnOnce() + Send + 'static,
  {
    tracing::debug!("Dispatching callback: {name}.");

    // Double box callback to avoid `STATUS_ACCESS_VIOLATION`.
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
        // If PostMessage fails, we need to clean up the callback
        let _ = Box::from_raw(callback_ptr);
        Err(anyhow::anyhow!("Failed to post message"))
      }
    }
  }

  /// Dispatches a callback and waits for it to complete.
  ///
  /// Returns Ok(R) if the callback completes successfully, or an
  /// Error if the callback panics or fails to send the result.
  pub async fn dispatch_and_wait<F, R>(
    &self,
    name: &str,
    callback: F,
  ) -> anyhow::Result<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    let name_clone = name.to_string();

    self.dispatch(name, move || {
      let result = callback();
      if sender.send(result).is_err() {
        tracing::error!(
          "Error sending dispatch result for {}",
          name_clone
        );
      }
    })?;

    match receiver.await {
      Ok(r) => Ok(r),
      Err(_) => Err(anyhow::anyhow!("Failed to receive dispatch result")),
    }
  }

  /// Shuts down the event loop gracefully.
  pub fn shutdown(&mut self) -> anyhow::Result<()> {
    tracing::info!("Shutting down event loop.");

    // Wait for the spawned thread to finish.
    if let Some(thread_handle) = self.thread_handle.take() {
      Platform::kill_message_loop(&thread_handle)?;

      thread_handle
        .join()
        .map_err(|_| anyhow::anyhow!("Thread join failed."))??;
    }

    Ok(())
  }

  /// Returns whether the event loop is still running.
  #[must_use]
  pub fn is_running(&self) -> bool {
    if let Some(ref handle) = self.thread_handle {
      !handle.is_finished()
    } else {
      false
    }
  }

  /// Returns the thread ID of the message loop thread.
  #[must_use]
  pub fn thread_id(&self) -> u32 {
    self.thread_id
  }

  /// Returns the window handle of the message loop.
  #[must_use]
  pub fn message_window_handle(&self) -> crate::WindowHandle {
    self.message_window_handle
  }

  /// Window procedure for handling messages.
  unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    // Can't match on a `LazyLock`, TODO: Possibly fixable using
    // `lazy_static`.
    if msg == *WM_DISPATCH_CALLBACK {
      // Convert the `wparam` fn pointer back to a double boxed function.
      let function: DispatchFn = Box::from_raw(wparam.0 as *mut _);
      function();
      return LRESULT(0);
    }
    match msg {
      WM_POWERBROADCAST => {
        #[allow(clippy::cast_possible_truncation)]
        match wparam.0 as u32 {
          // System is resuming from sleep/hibernation.
          PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => {
            IS_SYSTEM_SUSPENDED.store(false, Ordering::Relaxed);
          }
          // System is entering sleep/hibernation.
          PBT_APMSUSPEND => {
            IS_SYSTEM_SUSPENDED.store(true, Ordering::Relaxed);
          }
          _ => {}
        }

        LRESULT(0)
      }
      WM_DISPLAYCHANGE | WM_SETTINGCHANGE | WM_DEVICECHANGE => {
        // Ignore display change messages if the system hasn't fully
        // resumed from sleep.
        if !IS_SYSTEM_SUSPENDED.load(Ordering::Relaxed) {
          if let Err(err) = DisplayHook::handle_display_event(msg, wparam)
          {
            tracing::warn!(
              "Failed to handle display change message: {}",
              err
            );
          }
        }

        LRESULT(0)
      }
      WM_INPUT => {
        if let Err(err) = MouseHook::handle_mouse_input(wparam, lparam) {
          tracing::warn!("Failed to handle input message: {}", err);
        }

        LRESULT(0)
      }

      // `WM_QUIT` is handled for us by the message loop, so should be
      // forwarded along with other messages we don't care about.
      _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
  }

  /// Starts a message loop on the current thread.
  ///
  /// This function will block until the message loop is killed. Use
  /// `Platform::kill_message_loop` to terminate the message loop.
  fn run_message_loop() {
    let mut msg = MSG::default();

    loop {
      if unsafe { GetMessageW(&raw mut msg, None, 0, 0) }.as_bool() {
        unsafe {
          TranslateMessage(&raw const msg);
          DispatchMessageW(&raw const msg);
        }
      } else {
        break;
      }
    }
  }
}

impl Drop for EventLoop {
  fn drop(&mut self) {
    if let Err(err) = self.shutdown() {
      tracing::warn!("Failed to shut down event loop: {err}");
    }
  }
}
