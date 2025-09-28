use core::{
  fmt,
  mem::{self, ManuallyDrop},
};
use std::thread::ThreadId;

use crate::Dispatcher;

/// Binds a value to a specific event loop thread.
///
/// `ThreadBound<T>` wraps a value created on an event loop thread and
/// guarantees that all access and destruction of that value happens on
/// the same thread, using the provided [`Dispatcher`]. This allows the
/// wrapper to be used across threads (`Send + Sync`) even when `T` itself
/// is not thread-safe.
///
/// Inspired by:
/// - `threadbound::ThreadBound` <https://github.com/dtolnay/threadbound>
/// - `dispatch2::MainThreadBound` <https://github.com/madsmtm/objc2/tree/main/crates/dispatch2>
///
/// NOTE: Dropping the wrapper schedules the inner value to be dropped on
/// the event loop thread. If the event loop has already stopped, the drop
/// is skipped to avoid running `T`'s destructor on the wrong thread,
/// potentially leaking the value.
///
/// # Example usage
///
/// ```no_run
/// use wm_platform::{EventLoop, Dispatcher, ThreadBound};
///
/// # fn main() -> wm_platform::Result<()> {
/// let (event_loop, dispatcher) = EventLoop::new()?;
///
/// // Create the value on the event loop thread.
/// let bound = dispatcher.dispatch_sync(|| {
///   ThreadBound::new(String::from("hello"), dispatcher.clone())
/// })?;
///
/// // Access from any thread via the dispatcher.
/// let len = bound.with(|s| s.len())?;
/// assert_eq!(len, 5);
///
/// // Direct access only works on the original thread.
/// assert!(bound.get_ref().is_some());
///
/// drop(bound); // Drop is scheduled on the event loop thread.
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct ThreadBound<T> {
  value: ManuallyDrop<T>,
  thread_id: ThreadId,
  dispatcher: Dispatcher,
}

// SAFETY: Access to the inner value is only exposed via methods that
// dispatch the provided closure to the designated event loop thread using
// the stored `Dispatcher`. The inner value is also dropped on that thread
// synchronously in `Drop`, so the value never gets accessed from the wrong
// thread.
unsafe impl<T> Send for ThreadBound<T> {}
unsafe impl<T> Sync for ThreadBound<T> {}

impl<T> ThreadBound<T> {
  /// Create a new wrapper tied to the given event loop [`Dispatcher`].
  ///
  /// # Panics
  ///
  /// Panics if called from a thread different from the event loop thread
  /// referenced by the dispatcher.
  #[inline]
  pub fn new(inner: T, dispatcher: Dispatcher) -> Self {
    let thread_id = std::thread::current().id();

    // Ensure the dispatcher is tied to the same thread.
    assert_eq!(thread_id, dispatcher.thread_id());

    Self {
      value: ManuallyDrop::new(inner),
      thread_id,
      dispatcher,
    }
  }

  /// Returns `Some(&T)` if called on the original thread.
  ///
  /// Returns `None` if called from a different thread.
  #[inline]
  #[must_use]
  pub fn get_ref(&self) -> Option<&T> {
    if self.is_origin_thread() {
      Some(&self.value)
    } else {
      None
    }
  }

  /// Returns `Some(&mut T)` if called on the original thread.
  ///
  /// Returns `None` if called from a different thread.
  #[inline]
  #[must_use]
  pub fn get_mut(&mut self) -> Option<&mut T> {
    if self.is_origin_thread() {
      Some(&mut self.value)
    } else {
      None
    }
  }

  /// Consumes the wrapper and returns `Some(T)` if called on the
  /// original thread.
  ///
  /// Returns `None` if called from a different thread.
  #[inline]
  pub fn into_inner(self) -> Option<T> {
    if self.is_origin_thread() {
      // Prevent `Drop` from running.
      let mut this = ManuallyDrop::new(self);

      // SAFETY: `self` is consumed by this function, and wrapped in
      // `ManuallyDrop`, so the item's destructor is never run.
      Some(unsafe { ManuallyDrop::take(&mut this.value) })
    } else {
      None
    }
  }

  /// Execute a closure with `&T` on the event loop thread.
  ///
  /// Runs synchronously and returns the closure's result.
  #[inline]
  pub fn with<F, R>(&self, f: F) -> crate::Result<R>
  where
    F: Send + FnOnce(&T) -> R,
    R: Send,
  {
    self.dispatcher.dispatch_sync(|| f(&self.value))
  }

  /// Execute a closure with `&mut T` on the event loop thread.
  ///
  /// Runs synchronously and returns the closure's result.
  #[inline]
  #[allow(
    clippy::borrow_as_ptr,
    clippy::ptr_as_ptr,
    clippy::as_conversions
  )]
  pub fn with_mut<F, R>(&mut self, f: F) -> crate::Result<R>
  where
    F: Send + FnOnce(&mut T) -> R,
    R: Send,
  {
    let value_ptr =
      std::ptr::from_mut::<ManuallyDrop<T>>(&mut self.value) as usize;
    self.dispatcher.dispatch_sync(move || unsafe {
      // SAFETY: The closure executes on the event loop thread where the
      // value was created, and we only create a unique mutable reference.
      let value_mut: &mut T = &mut *(value_ptr as *mut T);
      f(value_mut)
    })
  }

  /// Returns `true` if called on the original thread.
  #[inline]
  #[must_use]
  pub fn is_origin_thread(&self) -> bool {
    std::thread::current().id() == self.thread_id
  }
}

impl<T> fmt::Debug for ThreadBound<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ThreadBound").finish_non_exhaustive()
  }
}

impl<T> Drop for ThreadBound<T> {
  #[allow(
    clippy::borrow_as_ptr,
    clippy::ptr_as_ptr,
    clippy::as_conversions,
    clippy::ref_as_ptr
  )]
  fn drop(&mut self) {
    if mem::needs_drop::<T>() {
      let value_ptr =
        std::ptr::from_mut::<ManuallyDrop<T>>(&mut self.value) as usize;

      let _ = self.dispatcher.dispatch_sync(move || unsafe {
        // SAFETY: The value is dropped on the event loop thread, which is
        // the same thread that it originated from (guaranteed by `new`).
        // Additionally, the value is never used again after this point.
        ManuallyDrop::drop(&mut *(value_ptr as *mut ManuallyDrop<T>));
      });
    }
  }
}
