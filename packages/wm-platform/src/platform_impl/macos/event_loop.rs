use std::{
  sync::{Arc, Mutex},
  thread::{self, ThreadId},
};

use anyhow::Context;
use objc2_core_foundation::{
  kCFRunLoopDefaultMode, CFRetained, CFRunLoop, CFRunLoopSource,
  CFRunLoopSourceContext,
};

/// Type alias for the callback function used with dispatches.
type DispatchFn = Box<Box<dyn FnOnce() + Send + 'static>>;

pub struct EventLoop {
  operations: Arc<Mutex<Vec<DispatchFn>>>,
  run_loop: Option<CFRetained<CFRunLoop>>,
  source: Option<CFRetained<CFRunLoopSource>>,
}

impl EventLoop {
  pub fn new() -> anyhow::Result<Self> {
    let operations = Arc::new(Mutex::new(Vec::new()));
    let operations_clone = Arc::clone(&operations);

    let (sender, receiver) = tokio::sync::oneshot::channel::<(
      CFRetained<CFRunLoopSource>,
      CFRetained<CFRunLoop>,
      ThreadId,
    )>();

    let thread_handle = thread::spawn(move || -> anyhow::Result<()> {
      let thread_id = thread::current().id();

      // Set up the `CFRunLoop` source.
      let (source_ptr, run_loop_ptr) =
        Self::setup_runloop(&operations_clone)?;

      // Send data back to main thread
      if sender.send((source_ptr, run_loop_ptr, thread_id)).is_err() {
        anyhow::bail!("Failed to send run loop data back to main thread");
      }

      // Run the run loop.
      Self::run_cf_runloop();

      tracing::info!("macOS event loop thread exiting.");
      Ok(())
    });

    let (source_ptr, run_loop_ptr, loop_thread_id) =
      receiver.blocking_recv()?;

    let event_loop = EventLoop {
      operations,
      run_loop: Some(run_loop_ptr.clone()),
      source: Some(source_ptr.clone()),
    };

    let dispatcher = EventLoopDispatcher {
      thread_id: loop_thread_id,
      running_flag: running_flag_weak,
      operations,
      source_ptr,
      run_loop_ptr,
    };

    Ok((event_loop, dispatcher))
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
      callback();
    }
  }

  fn run_cf_runloop() {
    CFRunLoop::run();
  }

  // Dispatch an operation from any thread
  fn dispatch(&self, operation: DispatchFn) {
    {
      let mut ops = self.operations.lock().unwrap();
      ops.push(operation);
    }

    // Signal the run loop source and wake up the run loop.
    if let (Some(source), Some(run_loop)) = (&self.source, &self.run_loop)
    {
      source.signal();
      run_loop.wake_up();
    }
  }
}
