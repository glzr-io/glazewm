use std::sync::{
  atomic::{AtomicBool, Ordering},
  mpsc, Arc,
};

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_core_foundation::{
  kCFRunLoopDefaultMode, CFRetained, CFRunLoop, CFRunLoopSource,
  CFRunLoopSourceContext,
};

use crate::{DispatchFn, Dispatcher};

#[derive(Clone)]
pub(crate) struct EventLoopSource {
  dispatch_tx: mpsc::Sender<Box<DispatchFn>>,
  source: CFRetained<CFRunLoopSource>,
  run_loop: CFRetained<CFRunLoop>,
  pub(crate) thread_id: std::thread::ThreadId,
}

impl EventLoopSource {
  pub(crate) fn send_dispatch_async<F>(
    &self,
    dispatch_fn: F,
  ) -> crate::Result<()>
  where
    F: FnOnce() + Send + 'static,
  {
    // TODO: Avoid duplicate check in `dispatch_sync`.
    if std::thread::current().id() == self.thread_id {
      dispatch_fn();
      return Ok(());
    }

    self.dispatch_tx.send(Box::new(dispatch_fn)).unwrap();

    // Signal the run loop source and wake up the run loop.
    self.source.signal();
    self.run_loop.wake_up();

    Ok(())
  }

  pub(crate) fn send_dispatch_sync<F>(
    &self,
    dispatch_fn: F,
  ) -> crate::Result<()>
  where
    F: FnOnce() + Send,
  {
    // SAFETY: This function is guaranteed to be used in a synchronous
    // context where the dispatch function will be executed before the
    // caller's stack frame is dropped. We transmute the lifetime to
    // satisfy the channel's `'static` requirement.
    let dispatch_fn_static = unsafe {
      std::mem::transmute::<
        Box<dyn FnOnce() + Send>,
        Box<dyn FnOnce() + Send + 'static>,
      >(Box::new(dispatch_fn))
    };

    self.send_dispatch_async(dispatch_fn_static)
  }

  pub(crate) fn send_stop(&self) -> crate::Result<()> {
    let (result_tx, result_rx) = std::sync::mpsc::channel();

    self.send_dispatch_sync(|| {
      let mtm = unsafe { MainThreadMarker::new_unchecked() };

      // Call `stop()` to mark the run loop for termination.
      let ns_app = NSApplication::sharedApplication(mtm);
      ns_app.stop(None);

      // `stop()` only takes effect after processing a subsequent UI event.
      // Post a dummy event so the application actually exits.
      unsafe { ns_app.abortModal() };

      let _ = result_tx.send(());
    })?;

    result_rx
      .recv_timeout(std::time::Duration::from_millis(3000))
      .map_err(crate::Error::ChannelRecv)
  }
}

// SAFETY: `CFRunLoop` and `CFRunLoopSource` are thread-safe Core
// Foundation types. The `objc2` bindings don't implement `Send + Sync`,
// but the underlying CF types are safe to send between threads.
unsafe impl Send for EventLoopSource {}
unsafe impl Sync for EventLoopSource {}

/// macOS-specific implementation of [`EventLoop`].
pub(crate) struct EventLoop {
  source: EventLoopSource,
  stopped: Arc<AtomicBool>,
}

impl EventLoop {
  pub fn new() -> crate::Result<(Self, Dispatcher)> {
    // Set up the `CFRunLoop` directly on the current thread.
    let source = Self::add_dispatch_source()?;

    let stopped = Arc::new(AtomicBool::new(false));
    let dispatcher =
      Dispatcher::new(Some(source.clone()), stopped.clone());

    Ok((
      Self {
        source: source.clone(),
        stopped,
      },
      dispatcher,
    ))
  }

  /// Runs the event loop.
  ///
  /// This method will block the current thread until the event loop is
  /// stopped.
  pub fn run(&self) -> crate::Result<()> {
    let mtm =
      MainThreadMarker::new().ok_or(crate::Error::NotMainThread)?;

    tracing::info!("Starting macOS event loop.");
    NSApplication::sharedApplication(mtm).run();

    tracing::info!("macOS event loop exiting.");
    Ok(())
  }

  /// Adds a source (`CFRunLoopSource`) for allowing dispatches to
  /// the current run loop.
  ///
  /// Can only be called on the main thread.
  pub(crate) fn add_dispatch_source() -> crate::Result<EventLoopSource> {
    let mtm =
      MainThreadMarker::new().ok_or(crate::Error::NotMainThread)?;

    // Ensure NSApplication is initialized on the main thread so
    // AppKit-based components (e.g. status bar items) are fully
    // functional. Use Accessory policy to avoid a Dock icon while
    // still allowing UI.
    let ns_app = NSApplication::sharedApplication(mtm);
    ns_app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let (dispatch_tx, dispatch_rx) = mpsc::channel();
    let dispatch_rx_ptr =
      Box::into_raw(Box::new(dispatch_rx)).cast::<std::ffi::c_void>();

    // Create `CFRunLoopSource` context.
    let mut context = CFRunLoopSourceContext {
      version: 0,
      info: dispatch_rx_ptr,
      retain: None,
      release: Some(Self::runloop_source_released_callback),
      copyDescription: None,
      equal: None,
      hash: None,
      schedule: None,
      cancel: None,
      perform: Some(Self::runloop_signaled_callback),
    };

    // Create the run loop source.
    let source =
      unsafe { CFRunLoopSource::new(None, 0, &raw mut context) }.ok_or(
        crate::Error::Platform(
          "Failed to create run loop source.".to_string(),
        ),
      )?;

    let run_loop =
      CFRunLoop::current().ok_or(crate::Error::EventLoopStopped)?;

    run_loop.add_source(Some(&source), unsafe { kCFRunLoopDefaultMode });

    Ok(EventLoopSource {
      dispatch_tx,
      source,
      run_loop,
      thread_id: std::thread::current().id(),
    })
  }

  // This function is called by the `CFRunLoopSource` when signaled.
  extern "C-unwind" fn runloop_signaled_callback(
    info: *mut std::ffi::c_void,
  ) {
    let operations =
      unsafe { &*(info as *const mpsc::Receiver<Box<DispatchFn>>) };

    for callback in operations.try_iter() {
      callback();
    }
  }

  // This function is called when the `CFRunLoopSource` is released.
  extern "C-unwind" fn runloop_source_released_callback(
    info: *const std::ffi::c_void,
  ) {
    // SAFETY: This pointer was created with `Box::into_raw` in
    // `add_dispatch_source`, so it can safely be converted back to a `Box`
    // and dropped.
    let _ = unsafe {
      Box::from_raw(info as *mut mpsc::Receiver<Box<DispatchFn>>)
    };
  }
}

impl Drop for EventLoop {
  fn drop(&mut self) {
    tracing::info!("Shutting down event loop.");

    // Stop the run loop if not already stopped.
    if !self.stopped.load(Ordering::SeqCst) {
      let _ = self.source.send_stop();
    }

    // Invalidate the runloop source to trigger its release callback. This
    // is thread-safe and is OK to call after the run loop is stopped.
    self.source.source.invalidate();
  }
}
