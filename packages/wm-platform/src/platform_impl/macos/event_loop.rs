use std::{
  sync::{Arc, Mutex},
  thread::{self, ThreadId},
};

use anyhow::Context;
use objc2_core_foundation::{
  kCFRunLoopDefaultMode, CFRetained, CFRunLoop, CFRunLoopSource,
  CFRunLoopSourceContext, Type,
};

use crate::platform_impl::EventLoopDispatcher;

/// Type alias for the callback function used with dispatches.
type DispatchFn = Box<Box<dyn FnOnce() + Send + 'static>>;

struct SendableCFRetained<T>(CFRetained<T>);

unsafe impl<T> Send for SendableCFRetained<T> {}

impl<T> SendableCFRetained<T> {
  /// Creates a new sendable wrapper around a CFRetained.
  ///
  /// # Safety
  ///
  /// The caller must ensure the wrapped type is only used on appropriate
  /// threads.
  fn new(retained: CFRetained<T>) -> Self {
    Self(retained)
  }

  /// Unwraps the CFRetained, consuming the wrapper.
  fn into_inner(self) -> CFRetained<T> {
    self.0
  }
}

pub struct EventLoop {
  operations: Arc<Mutex<Vec<DispatchFn>>>,
  run_loop: Option<CFRetained<CFRunLoop>>,
  source: Option<CFRetained<CFRunLoopSource>>,
}

impl EventLoop {
  pub fn new() -> anyhow::Result<(Self, EventLoopDispatcher)> {
    let operations = Arc::new(Mutex::new(Vec::new()));
    let operations_clone = Arc::clone(&operations);

    let (sender, receiver) = tokio::sync::oneshot::channel::<(
      SendableCFRetained<CFRunLoopSource>,
      SendableCFRetained<CFRunLoop>,
      ThreadId,
    )>();

    let thread_handle = thread::spawn(move || -> anyhow::Result<()> {
      let thread_id = thread::current().id();

      // Set up the `CFRunLoop` source.
      let (source_ptr, run_loop_ptr) =
        Self::setup_runloop(&operations_clone)?;

      // Send data back to main thread
      if sender
        .send((
          SendableCFRetained::new(source_ptr.retain()),
          SendableCFRetained::new(run_loop_ptr.retain()),
          thread_id,
        ))
        .is_err()
      {
        anyhow::bail!("Failed to send run loop data back to main thread");
      }

      // Run the run loop.
      Self::run_cf_runloop();

      tracing::info!("macOS event loop thread exiting.");
      Ok(())
    });

    let (source_ptr, run_loop_ptr, loop_thread_id) =
      receiver.blocking_recv()?;

    // Unwrap the CFRetained objects.
    let source = source_ptr.into_inner();
    let run_loop = run_loop_ptr.into_inner();

    let event_loop = EventLoop {
      operations: operations.clone(),
      run_loop: Some(run_loop.clone()),
      source: Some(source.clone()),
    };

    let dispatcher =
      EventLoopDispatcher::new(operations, Some(run_loop), Some(source));

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
