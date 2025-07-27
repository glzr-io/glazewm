use std::sync::{Arc, Mutex};

use objc2_core_foundation::{CFRetained, CFRunLoop, CFRunLoopSource};

/// Type alias for the callback function used with dispatches.
type DispatchFn = Box<Box<dyn FnOnce() + Send + 'static>>;

#[derive(Clone)]
pub struct EventLoopDispatcher {
  operations: Arc<Mutex<Vec<DispatchFn>>>,
  run_loop: Option<CFRetained<CFRunLoop>>,
  source: Option<CFRetained<CFRunLoopSource>>,
}

// Safety: `CFRunLoop` and `CFRunLoopSource` are thread-safe Core
// Foundation types. The `objc2` bindings don't implement `Send + Sync`,
// but the underlying CF types are safe to send between threads.
unsafe impl Send for EventLoopDispatcher {}
unsafe impl Sync for EventLoopDispatcher {}

impl EventLoopDispatcher {
  pub fn new(
    operations: Arc<Mutex<Vec<DispatchFn>>>,
    run_loop: Option<CFRetained<CFRunLoop>>,
    source: Option<CFRetained<CFRunLoopSource>>,
  ) -> Self {
    Self {
      operations,
      run_loop,
      source,
    }
  }

  pub fn run<F>(&self, callback: F) -> anyhow::Result<()>
  where
    F: FnOnce() + Send + 'static,
  {
    let callback: DispatchFn = Box::new(Box::new(callback));

    {
      let mut ops = self.operations.lock().unwrap();
      ops.push(callback);
    }

    // Signal the run loop source and wake up the run loop.
    if let (Some(source), Some(run_loop)) = (&self.source, &self.run_loop)
    {
      source.signal();
      run_loop.wake_up();
    }

    Ok(())
  }
}
