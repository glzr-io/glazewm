use std::sync::{atomic::AtomicBool, mpsc, Arc};

use anyhow::Context;
use dispatch2::DispatchQueue;
use objc2::MainThreadMarker;
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
}

impl EventLoopSource {
  pub fn send_dispatch(
    &self,
    dispatch_fn: Box<DispatchFn>,
  ) -> crate::Result<()> {
    // self
    //   .dispatch_tx
    //   .send(dispatch_fn)
    //   .map_err(|err| crate::Error::ChannelSend(err.to_string()))?;

    // self.source.signal();
    // self.run_loop.wake_up();

    DispatchQueue::main().exec_sync(|| {
      dispatch_fn();
    });

    // dispatch2::run_on_main(move || {
    // });

    Ok(())
  }

  pub fn send_stop(&self) -> crate::Result<()> {
    self.run_loop.stop();
    Ok(())
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
}

impl EventLoop {
  pub fn new() -> crate::Result<(Self, Dispatcher)> {
    // Set up the `CFRunLoop` directly on the current thread.
    let source = Self::add_dispatch_source()?;

    let stopped = Arc::new(AtomicBool::new(false));
    let dispatcher = Dispatcher::new(Some(source.clone()), stopped);

    Ok((
      Self {
        source: source.clone(),
      },
      dispatcher,
    ))
  }

  /// Runs the event loop.
  ///
  /// This method will block the current thread until the event loop is
  /// stopped.
  pub fn run(&self) -> crate::Result<()> {
    tracing::info!("Starting macOS event loop.");
    CFRunLoop::run();
    tracing::info!("macOS event loop exiting.");
    Ok(())
  }

  /// Adds a source (`CFRunLoopSource`) for allowing dispatches to
  /// the current run loop.
  ///
  /// Can only be called on the main thread.
  pub(crate) fn add_dispatch_source() -> crate::Result<EventLoopSource> {
    MainThreadMarker::new().ok_or(crate::Error::NotMainThread)?;

    let (dispatch_tx, dispatch_rx) = mpsc::channel();
    let dispatch_rx_ptr =
      Box::into_raw(Box::new(dispatch_rx)).cast::<std::ffi::c_void>();

    // Create `CFRunLoopSource` context.
    let mut context = CFRunLoopSourceContext {
      version: 0,
      info: dispatch_rx_ptr,
      retain: None,
      release: None,
      copyDescription: None,
      equal: None,
      hash: None,
      schedule: None,
      cancel: None,
      perform: Some(Self::perform_operations),
    };

    // Create the run loop source.
    let source =
      unsafe { CFRunLoopSource::new(None, 0, &raw mut context) }
        .context("Failed to create run loop source.")?;

    let run_loop =
      CFRunLoop::current().context("Failed to get current run loop.")?;

    run_loop.add_source(Some(&source), unsafe { kCFRunLoopDefaultMode });

    Ok(EventLoopSource {
      dispatch_tx,
      source,
      run_loop,
    })
  }

  // This function is called by the `CFRunLoopSource` when signaled.
  extern "C-unwind" fn perform_operations(info: *mut std::ffi::c_void) {
    let operations =
      unsafe { &*(info as *const mpsc::Receiver<Box<DispatchFn>>) };

    for callback in operations.try_iter() {
      tracing::info!("Running callback from event loop.");
      callback();
    }
  }
}

impl Drop for EventLoop {
  fn drop(&mut self) {
    tracing::info!("Shutting down event loop.");

    // Removing the added `CFRunLoopSource` prior to stopping the run loop
    // is not necessary.
    self.source.run_loop.stop();
  }
}
