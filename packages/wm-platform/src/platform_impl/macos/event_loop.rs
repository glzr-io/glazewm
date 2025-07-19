use std::{
  sync::{Arc, Mutex},
  time::Duration,
};

use core_foundation::runloop::{
  kCFRunLoopDefaultMode, CFRunLoop, CFRunLoopAddSource, CFRunLoopSource,
  CFRunLoopSourceContext, CFRunLoopSourceSignal, CFRunLoopWakeUp,
};
use objc2_app_kit::{NSApplication, NSPoint, NSRect, NSWindow};
use objc2_foundation::{
  NSDate, NSDefaultRunLoopMode, NSRunLoop, NSString,
};
use tokio::sync::mpsc;

// Shared state between tokio and run loop thread
struct RunLoopDispatcher {
  operations: Arc<Mutex<Vec<WindowOperation>>>,
  run_loop: Option<CFRunLoop>,
  source: Option<CFRunLoopSource>,
}

impl RunLoopDispatcher {
  fn new() -> Self {
    Self {
      operations: Arc::new(Mutex::new(Vec::new())),
      run_loop: None,
      source: None,
    }
  }

  fn setup_runloop(&mut self) {
    let operations_ptr =
      Arc::as_ptr(&self.operations) as *mut std::ffi::c_void;

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

    // Create the run loop source
    let source = unsafe {
      CFRunLoopSource::create(std::ptr::null(), 0, &mut context)
    };

    let current_loop = CFRunLoop::get_current();

    // Add source to run loop
    unsafe {
      CFRunLoopAddSource(
        current_loop.clone(),
        source.clone(),
        kCFRunLoopDefaultMode,
      );
    }

    self.run_loop = Some(current_loop);
    self.source = Some(source);
  }

  // This function is called by the CFRunLoopSource when signaled
  extern "C" fn perform_operations(info: *const std::ffi::c_void) {
    let operations =
      unsafe { &*(info as *const Mutex<Vec<WindowOperation>>) };

    let mut ops = operations.lock().unwrap();
    let operations_to_process: Vec<WindowOperation> =
      ops.drain(..).collect();
    drop(ops); // Release the lock

    for op in operations_to_process {
      Self::execute_operation(op);
    }
  }

  fn execute_operation(op: WindowOperation) {
    match op {
            // …
        }
  }

  // Dispatch an operation from any thread
  fn dispatch(&self, operation: WindowOperation) {
    {
      let mut ops = self.operations.lock().unwrap();
      ops.push(operation);
    }

    // Signal the run loop source and wake up the run loop
    if let (Some(source), Some(run_loop)) = (&self.source, &self.run_loop)
    {
      unsafe {
        CFRunLoopSourceSignal(source.clone());
        CFRunLoopWakeUp(run_loop.clone());
      }
    }
  }
}
