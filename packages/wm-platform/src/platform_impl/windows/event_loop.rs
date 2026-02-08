use std::{
  cell::RefCell,
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
  },
  thread::{self, JoinHandle, ThreadId},
};

use windows::{
  core::w,
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
      DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
      PostMessageW, PostThreadMessageW, RegisterWindowMessageW,
      SendMessageW, TranslateMessage, MSG, WM_QUIT,
    },
  },
};

use crate::{DispatchFn, Dispatcher, WindowId};

thread_local! {
  /// Custom message ID for dispatching closures to be run on the event
  /// loop thread.
  ///
  /// `WPARAM` contains a `Box<Box<dyn FnOnce()>>` that must be retrieved
  /// with `Box::from_raw`. `LPARAM` is unused.
  ///
  /// This message is sent using `PostMessageW` and handled in
  /// [`EventLoop::window_proc`].
  static WM_DISPATCH_CALLBACK: u32 = unsafe { RegisterWindowMessageW(w!("GlazeWM:Dispatch")) };
}

/// A callback that pre-processes Windows messages received by the event
/// loop's hidden message window.
///
/// Returns `Some(LRESULT)` if the message was handled, or `None` to pass
/// the message to subsequent callbacks or the default handler.
pub type WndProcCallback =
  dyn Fn(HWND, u32, WPARAM, LPARAM) -> Option<LRESULT> + Send + 'static;

thread_local! {
  /// Registered callbacks that pre-process messages in the event
  /// loop's window procedure.
  ///
  /// Keyed by a unique callback ID for later deregistration.
  static WNDPROC_CALLBACKS: RefCell<HashMap<usize, Box<WndProcCallback>>> =
    RefCell::new(HashMap::new());
}

#[derive(Clone)]
pub(crate) struct EventLoopSource {
  pub(crate) message_window_handle: WindowId,
  pub(crate) thread_id: ThreadId,
  win32_thread_id: u32,
  next_callback_id: Arc<AtomicUsize>,
}

impl EventLoopSource {
  pub(crate) fn send_dispatch_async<F>(
    &self,
    dispatch_fn: F,
  ) -> crate::Result<()>
  where
    F: FnOnce() + Send + 'static,
  {
    // Double box the callback to get a thin raw pointer and to avoid
    // `STATUS_ACCESS_VIOLATION` on Windows.
    // Ref: https://github.com/tauri-apps/tao/blob/dev/src/platform_impl/windows/event_loop.rs#L596
    let dispatch_fn: Box<Box<DispatchFn>> = Box::new(Box::new(dispatch_fn));

    // Leak to a raw pointer to then be passed as `WPARAM` in the message.
    let callback_ptr = Box::into_raw(dispatch_fn);

    unsafe {
      if PostMessageW(
        HWND(self.message_window_handle.0),
        WM_DISPATCH_CALLBACK.with(|v| *v),
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

  /// Synchronously dispatches a closure to the event loop thread.
  ///
  /// Uses `SendMessageW`, which blocks the calling thread until the
  /// event loop thread's window procedure processes the message and
  /// calls the closure.
  ///
  /// # Safety invariant
  ///
  /// Because `SendMessageW` blocks until the closure completes, closures
  /// with non-`'static` captures are safe: the caller's stack frame is
  /// guaranteed to outlive the closure execution.
  pub(crate) fn send_dispatch_sync<F>(
    &self,
    dispatch_fn: F,
  ) -> crate::Result<()>
  where
    F: FnOnce() + Send,
  {
    let dispatch_fn: Box<Box<dyn FnOnce() + Send>> =
      Box::new(Box::new(dispatch_fn));
    let callback_ptr = Box::into_raw(dispatch_fn);

    // SAFETY: `SendMessageW` blocks the calling thread until the window
    // procedure on the event loop thread processes the message and calls
    // the closure. This guarantees the closure's captures remain alive
    // for the entire duration, even with non-'static lifetimes.
    unsafe {
      SendMessageW(
        HWND(self.message_window_handle.0),
        WM_DISPATCH_CALLBACK.with(|v| *v),
        WPARAM(callback_ptr as _),
        LPARAM(0),
      );
    }

    Ok(())
  }

  pub(crate) fn send_stop(&self) -> crate::Result<()> {
    unsafe {
      PostThreadMessageW(
        self.win32_thread_id,
        WM_QUIT,
        WPARAM(0),
        LPARAM(0),
      )
    }
    .map_err(|_| {
      crate::Error::WindowMessage(
        "Failed to post quit message".to_string(),
      )
    })
  }

  pub(crate) fn register_wndproc_callback(
    &self,
    callback: Box<WndProcCallback>,
  ) -> crate::Result<usize> {
    let id = self.next_callback_id.fetch_add(1, Ordering::Relaxed);

    // The callback is installed asynchronously on the event loop thread.
    self.send_dispatch_async(move || {
      WNDPROC_CALLBACKS.with(|cbs| {
        cbs.borrow_mut().insert(id, callback);
      });
    })?;

    Ok(id)
  }

  pub(crate) fn deregister_wndproc_callback(
    &self,
    id: usize,
  ) -> crate::Result<()> {
    self.send_dispatch_async(move || {
      WNDPROC_CALLBACKS.with(|cbs| {
        cbs.borrow_mut().remove(&id);
      });
    })
  }
}

/// Windows-specific implementation of [`EventLoop`].
pub(crate) struct EventLoop {
  message_window_handle: WindowId,
  thread_handle: Option<JoinHandle<crate::Result<()>>>,
  win32_thread_id: u32,
}

impl EventLoop {
  pub fn new() -> crate::Result<(Self, Dispatcher)> {
    let (sender, receiver) =
      tokio::sync::oneshot::channel::<(WindowId, u32, ThreadId)>();

    let thread_handle = thread::spawn(move || -> crate::Result<()> {
      // Create a hidden message window on the current thread.
      let window_handle =
        super::Platform::create_message_window(Some(Self::window_proc))?;

      let win32_thread_id = unsafe { GetCurrentThreadId() };
      let rust_thread_id = thread::current().id();

      // Send the window handle and thread ID back to the main thread. Will
      // only fail if the receiver was closed, which would be due to the
      // main thread erroring.
      if sender
        .send((WindowId(window_handle), win32_thread_id, rust_thread_id))
        .is_err()
      {
        unsafe { DestroyWindow(HWND(window_handle)) }?;
        return Err(crate::Error::Platform(
          "Failed to send window handle back to main thread, channel was closed.".to_string(),
        ));
      }

      // Run the message loop. This will block until `WM_QUIT` is
      // dispatched.
      Self::run_message_loop();

      tracing::info!("Event loop thread exiting.");
      unsafe { DestroyWindow(HWND(window_handle)) }?;

      Ok(())
    });

    // Wait for the window handle and thread ID.
    let (window_handle, win32_thread_id, rust_thread_id) =
      receiver.blocking_recv()?;

    let event_loop = EventLoop {
      message_window_handle: window_handle,
      thread_handle: Some(thread_handle),
      win32_thread_id,
    };

    let event_loop_source = EventLoopSource {
      message_window_handle: window_handle,
      thread_id: rust_thread_id,
      win32_thread_id,
      next_callback_id: Arc::new(AtomicUsize::new(0)),
    };

    let stopped = Arc::new(AtomicBool::new(false));
    let dispatcher = Dispatcher::new(Some(event_loop_source), stopped);

    Ok((event_loop, dispatcher))
  }

  /// Runs the event loop, blocking until shutdown.
  ///
  /// This will block the current thread until the event loop is
  /// stopped.
  pub fn run(mut self) -> crate::Result<()> {
    tracing::info!("Starting Windows event loop.");

    // Join the thread to wait for completion
    if let Some(thread_handle) = self.thread_handle.take() {
      thread_handle
        .join()
        .map_err(|_| {
          crate::Error::Thread("Event loop thread panicked".to_string())
        })?
        .map_err(|e| crate::Error::Platform(e.to_string()))?;
    }

    tracing::info!("Windows event loop exiting.");
    Ok(())
  }

  /// Shuts down the event loop gracefully.
  pub fn shutdown(&mut self) -> crate::Result<()> {
    tracing::info!("Shutting down event loop.");

    // Wait for the spawned thread to finish.
    if let Some(thread_handle) = self.thread_handle.take() {
      super::Platform::kill_message_loop(&thread_handle)?;

      thread_handle.join().map_err(|_| {
        crate::Error::Thread("Thread join failed.".to_string())
      })??;
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
    self.win32_thread_id
  }

  /// Returns the window handle of the message loop.
  #[must_use]
  pub fn message_window_handle(&self) -> WindowId {
    self.message_window_handle
  }

  /// Window procedure for handling messages.
  unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    let dispatch_msg = WM_DISPATCH_CALLBACK.with(|v| *v);

    // Handle dispatch callbacks first.
    if msg == dispatch_msg {
      // Convert the `WPARAM` fn pointer back to a double-boxed function.
      let dispatch_fn: Box<Box<dyn FnOnce() + Send>> =
        Box::from_raw(wparam.0 as *mut _);
      dispatch_fn();
      return LRESULT(0);
    }

    // Let registered callbacks pre-process the message.
    let handled = WNDPROC_CALLBACKS.with(|cbs| {
      for callback in cbs.borrow().values() {
        if let Some(result) = callback(hwnd, msg, wparam, lparam) {
          return Some(result);
        }
      }
      None
    });

    if let Some(result) = handled {
      return result;
    }

    // `WM_QUIT` is handled by the message loop and should be forwarded
    // along with other messages.
    DefWindowProcW(hwnd, msg, wparam, lparam)
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
