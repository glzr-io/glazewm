use std::sync::{Arc, Mutex};

use anyhow::Context;
use objc2_core_foundation::{CFRetained, CFRunLoop, CFRunLoopSource};

/// Type alias for the closure used with dispatches.
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

  /// Dispatches a closure to be executed on the event loop thread.
  ///
  /// This is a fire-and-forget operation that schedules the closure to
  /// run asynchronously on the thread. The calling thread does not
  /// wait for the closure to complete and no result is returned.
  ///
  /// Returns `Ok(())` if the closure was successfully queued.
  pub fn run<F>(&self, dispatch_fn: F) -> anyhow::Result<()>
  where
    F: FnOnce() + Send + 'static,
  {
    let dispatch_fn: DispatchFn = Box::new(Box::new(dispatch_fn));

    {
      let mut ops = self.operations.lock().unwrap();
      ops.push(dispatch_fn);
    }

    // Signal the run loop source and wake up the run loop.
    if let (Some(source), Some(run_loop)) = (&self.source, &self.run_loop)
    {
      source.signal();
      run_loop.wake_up();
    }

    Ok(())
  }

  /// Dispatches a closure to be executed on the event loop thread and
  /// blocks until it completes, returning its result.
  ///
  /// This method synchronously executes the closure on the thread. The
  /// calling thread will block until the closure finishes executing.
  ///
  /// Returns a result containing the closure's return value if successful.
  pub fn run_sync<F, R>(&self, dispatch_fn: F) -> anyhow::Result<R>
  where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
  {
    let (res_tx, res_rx) = std::sync::mpsc::channel();

    self.run(move || {
      let res = dispatch_fn();

      if res_tx.send(res).is_err() {
        tracing::error!("Failed to send closure result.");
      }
    })?;

    res_rx.recv().context("Failed to receive closure result.")
  }
}
