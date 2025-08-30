use std::sync::{Arc, Mutex};

use anyhow::Context;
use objc2::{msg_send, ClassType};
use objc2_core_foundation::{
  kCFRunLoopDefaultMode, CFRetained, CFRunLoop, CFRunLoopSource,
  CFRunLoopSourceContext,
};
use objc2_foundation::NSThread;

use crate::platform_impl::EventLoopDispatcher;

/// Type alias for the callback function used with dispatches.
type DispatchFn = Box<Box<dyn FnOnce() + Send + 'static>>;

pub struct EventLoopInstaller {
  operations: Arc<Mutex<Vec<DispatchFn>>>,
}

impl EventLoopInstaller {
  pub fn new() -> crate::Result<(Self, EventLoopDispatcher)> {
    let operations = Arc::new(Mutex::new(Vec::new()));
    
    // Create a dispatcher that will be set up when installed
    let dispatcher = EventLoopDispatcher::new(
      operations.clone(),
      None, // Will be set when installed
      None, // Will be set when installed
    );

    Ok((Self { operations }, dispatcher))
  }

  /// Install on the main thread (macOS only).
  ///
  /// This method integrates with the existing CFRunLoop on the main thread.
  /// It must be called from the main thread.
  pub fn install(self) -> crate::Result<()> {
    let is_main_thread: bool =
      unsafe { msg_send![NSThread::class(), isMainThread] };

    // Verify we're on the main thread.
    if !is_main_thread {
      return Err(crate::Error::NotMainThread);
    }

    let (_source, _run_loop) = Self::setup_runloop(&self.operations)?;

    tracing::info!("EventLoopInstaller installed on main thread.");
    Ok(())
  }

  fn setup_runloop(
    operations: &Arc<Mutex<Vec<DispatchFn>>>,
  ) -> crate::Result<(CFRetained<CFRunLoopSource>, CFRetained<CFRunLoop>)>
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
      callback();
    }
  }
}