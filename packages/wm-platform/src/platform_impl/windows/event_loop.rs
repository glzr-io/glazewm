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

/// Custom message ID for our dispatch mechanism
static WM_DISPATCH_CALLBACK: LazyLock<u32> = LazyLock::new(|| unsafe {
  RegisterWindowMessageA(s!("Glaze:Dispatch"))
});

/// Whether the system is currently sleeping/hibernating.
///
/// For use with window procedure.
static IS_SYSTEM_SUSPENDED: AtomicBool = AtomicBool::new(false);

/// Type used for cleanup functions
type CleanupFn = Box<dyn FnOnce() -> anyhow::Result<()>>;

thread_local! {
  /// Thread-local storage for cleanup functions
  static CLEANUP_FUNCTIONS: RefCell<Vec<CleanupFn>> = RefCell::new(Vec::new());
}

/// Type alias for the callback function used with dispatches
type EventCallback = Box<Box<dyn FnOnce() + Send + 'static>>;

/// The install and stop functions for an installable component.
pub struct Installable<I, S>
where
  I: FnOnce() -> anyhow::Result<()> + Send + 'static,
  S: FnOnce() -> anyhow::Result<()> + Send + 'static,
{
  /// Called on the event loop thread immediately to install the
  /// component.
  pub installer: I,
  /// Stored on the event loop thread and gets called before the event
  /// loop is exits.
  pub stop: S,
}

/// Objects related to the event loop running on the event thread. Will
/// shutdown the event loop when dropped.
///
/// Callbacks can be dispatched to the event loop thread using `dispatch`,
/// and components installed via `install`.
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
      let hwnd =
        super::Platform::create_message_window(Some(Self::window_proc))?;
      let thread_id = unsafe { GetCurrentThreadId() };

      // Send the window handle and thread ID back to the main thread. Will
      // only fail if the reciever was closed, which would be due to
      // the main thread erroring - so just bail.
      if sender.send((hwnd, thread_id)).is_err() {
        unsafe { DestroyWindow(HWND(hwnd)) }?;
        bail!("Failed to send window handle back to main thread, channel was closed");
      }

      // Run the message loop. This will block until `WM_QUIT` is
      // dispatched.
      Self::run_message_loop();

      tracing::info!("Event thread exiting");

      // Run cleanup functions from any installed components.
      let fns = CLEANUP_FUNCTIONS.with(|fns| fns.replace(vec![]));
      for cleanup_fn in fns {
        if let Err(err) = cleanup_fn() {
          eprintln!("Cleanup function failed: {err}");
        }
      }

      unsafe { DestroyWindow(HWND(hwnd)) }?;

      Ok(())
    });

    // Wait for the window handle and thread ID
    let (hwnd, thread_id) = receiver.blocking_recv()?;

    Ok(EventLoop {
      message_window_handle: hwnd,
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
    tracing::debug!("Dispatching callback: {name}");

    // Needs to be double boxed to avoid a `STATUS_ACCESS_VIOLATION`. See
    // Tau's implementation: https://github.com/tauri-apps/tao/blob/dev/src/platform_impl/windows/event_loop.rs#L596
    let boxed_callback = Box::new(callback);
    let box2: EventCallback = Box::new(boxed_callback);
    // Leak to a raw pointer to be passed as WPARAM in the message.
    let callback_ptr = Box::into_raw(box2);

    // Post a message with the callback pointer as wParam
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

  /// Installs a component on the event loop thread.
  pub async fn install<I, S>(
    &mut self,
    name: &str,
    installable: Installable<I, S>,
  ) -> anyhow::Result<()>
  where
    I: FnOnce() -> anyhow::Result<()> + Send + 'static,
    S: FnOnce() -> anyhow::Result<()> + Send + 'static + Sync,
  {
    // Dispatch the installer function to the message loop thread
    self
      .dispatch_and_wait(name, move || -> anyhow::Result<()> {
        // Run the installer immediately
        (installable.installer)()?;

        // Queue the cleanup function
        CLEANUP_FUNCTIONS.with(|fns| {
          fns.borrow_mut().push(Box::new(installable.stop));
        });

        Ok(())
      })
      .await??;

    Ok(())
  }

  /// Shuts down the event loop gracefully
  pub fn shutdown(&mut self) -> anyhow::Result<()> {
    tracing::info!("Shutting down event loop");
    unsafe {
      // Post WM_QUIT to terminate the message loop
      if PostThreadMessageW(
        self.thread_id,
        WM_QUIT,
        WPARAM::default(),
        LPARAM::default(),
      )
      .is_err()
      {
        return Err(anyhow::anyhow!("Failed to post quit message"));
      }
    }

    if let Some(handle) = self.thread_handle.take() {
      // Wait for the thread to finish
      if let Err(err) = handle.join() {
        let msg = match err.downcast_ref::<&'static str>() {
          Some(s) => *s,
          None => match err.downcast_ref::<String>() {
            Some(s) => &s[..],
            None => "Unknown error",
          },
        };

        Err(anyhow::anyhow!("Event loop thread panicked: {}", msg))
      } else {
        Ok(())
      }
    } else {
      Ok(())
    }
  }

  /// Returns true if the event loop is still running
  #[must_use]
  pub fn is_running(&self) -> bool {
    if let Some(ref handle) = self.thread_handle {
      !handle.is_finished()
    } else {
      false
    }
  }

  /// Returns the thread ID of the message loop thread
  #[must_use]
  pub fn thread_id(&self) -> u32 {
    self.thread_id
  }

  /// Returns the window handle of the message loop
  #[must_use]
  pub fn message_window_handle(&self) -> crate::WindowHandle {
    self.message_window_handle
  }

  /// Window procedure for handling messages
  unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    // Can't match on a LazyLock, TODO: Possibly fixable using lazy_static?
    if msg == *WM_DISPATCH_CALLBACK {
      // Convert the wparam fn pointer back to a double boxed function.
      let function: EventCallback = Box::from_raw(wparam.0 as *mut _);
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

  /// Runs the Win32 message loop
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
      eprintln!("Failed to shut down event loop: {err}");
    }
  }
}
