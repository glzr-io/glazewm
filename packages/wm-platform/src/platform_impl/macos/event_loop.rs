use std::sync::{Arc, Mutex};

use anyhow::Context;
use objc2_core_foundation::{
  kCFRunLoopDefaultMode, CFRetained, CFRunLoop, CFRunLoopSource,
  CFRunLoopSourceContext,
};

use crate::{DispatchFn, Dispatcher};

#[derive(Clone)]
pub(crate) struct EventLoopSource {
  source: CFRetained<CFRunLoopSource>,
  run_loop: CFRetained<CFRunLoop>,
}

impl EventLoopSource {
  pub fn queue_dispatch(&self) {
    self.source.signal();
    self.run_loop.wake_up();
  }
}

// SAFETY: `CFRunLoop` and `CFRunLoopSource` are thread-safe Core
// Foundation types. The `objc2` bindings don't implement `Send + Sync`,
// but the underlying CF types are safe to send between threads.
unsafe impl Send for EventLoopSource {}
unsafe impl Sync for EventLoopSource {}

/// macOS-specific implementation of [`EventLoop`].
pub(crate) struct EventLoop {
  operations: Arc<Mutex<Vec<DispatchFn>>>,
  source: EventLoopSource,
}

impl EventLoop {
  pub fn new() -> anyhow::Result<(Self, Dispatcher)> {
    // TODO: Need to verify we're on the main thread.

    let operations = Arc::new(Mutex::new(Vec::new()));

    // Set up the `CFRunLoop` directly on the current thread.
    let source = Self::create_run_loop(&operations)?;

    let event_loop = EventLoop {
      operations: operations.clone(),
      source: source.clone(),
    };

    let dispatcher = Dispatcher::new(operations, Some(source));

    Ok((event_loop, dispatcher))
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

  pub(crate) fn create_run_loop(
    operations: &Arc<Mutex<Vec<DispatchFn>>>,
  ) -> anyhow::Result<EventLoopSource> {
    let operations_ptr = Arc::as_ptr(operations) as *mut std::ffi::c_void;

    // Create `CFRunLoopSource` context.
    let mut context = CFRunLoopSourceContext {
      version: 0,
      info: operations_ptr,
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

    Ok(EventLoopSource { source, run_loop })
  }

  // This function is called by the `CFRunLoopSource` when signaled.
  extern "C-unwind" fn perform_operations(info: *mut std::ffi::c_void) {
    let operations = unsafe { &*(info as *const Mutex<Vec<DispatchFn>>) };

    let callbacks = {
      let mut ops = operations.lock().unwrap();
      ops.drain(..).collect::<Vec<_>>()
    };

    for callback in callbacks {
      println!("Running callback from event loop.");
      callback();
    }
  }
}

impl Drop for EventLoop {
  fn drop(&mut self) {
    tracing::info!("Shutting down event loop.");

    self
      .source
      .run_loop
      .remove_source(Some(&self.source.source), None);

    self.source.run_loop.stop();
  }
}
