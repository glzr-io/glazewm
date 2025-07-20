use std::sync::{Arc, Mutex};

use anyhow::Context;
use objc2_core_foundation::{
  kCFRunLoopDefaultMode, CFRetained, CFRunLoop, CFRunLoopSource,
  CFRunLoopSourceContext,
};

use crate::platform_impl::EventLoopDispatcher;

/// Type alias for the callback function used with dispatches.
type DispatchFn = Box<Box<dyn FnOnce() + Send + 'static>>;

pub struct EventLoop {
  operations: Arc<Mutex<Vec<DispatchFn>>>,
  run_loop: CFRetained<CFRunLoop>,
  source: CFRetained<CFRunLoopSource>,
}

impl EventLoop {
  pub fn new() -> anyhow::Result<(Self, EventLoopDispatcher)> {
    // TODO: Need to verify we're on the main thread.

    let operations = Arc::new(Mutex::new(Vec::new()));

    // Set up the `CFRunLoop` directly on the current thread.
    let (source, run_loop) = Self::setup_runloop(&operations)?;

    let event_loop = EventLoop {
      operations: operations.clone(),
      run_loop: run_loop.clone(),
      source: source.clone(),
    };

    let dispatcher =
      EventLoopDispatcher::new(operations, Some(run_loop), Some(source));

    Ok((event_loop, dispatcher))
  }

  /// Runs the event loop.
  ///
  /// This method will block the current thread until the event loop is
  /// stopped.
  pub fn run(&self) {
    tracing::info!("Starting macOS event loop.");
    CFRunLoop::run();
    tracing::info!("macOS event loop exiting.");
  }

  fn setup_runloop(
    operations: &Arc<Mutex<Vec<DispatchFn>>>,
  ) -> anyhow::Result<(CFRetained<CFRunLoopSource>, CFRetained<CFRunLoop>)>
  {
    let operations_ptr = Arc::as_ptr(operations) as *mut std::ffi::c_void;

    // Create CFRunLoopSource context
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
    let source = unsafe { CFRunLoopSource::new(None, 0, &mut context) }
      .context("Failed to create run loop source.")?;

    let current_loop =
      CFRunLoop::current().context("Failed to get current run loop.")?;

    current_loop
      .add_source(Some(&source), unsafe { kCFRunLoopDefaultMode });

    Ok((source, current_loop))
  }

  // This function is called by the CFRunLoopSource when signaled.
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
