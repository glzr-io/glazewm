use std::{collections::HashMap, marker::PhantomData, sync::Mutex};

use crate::Dispatcher;

// Using `Box<dyn std::any::Any>` to store heterogeneous types.
type MainThreadStorage = HashMap<u64, Box<dyn std::any::Any>>;

// Global storage that lives on the main thread.
thread_local! {
  static MAIN_THREAD_STORAGE: Mutex<MainThreadStorage> = Mutex::new(HashMap::new());
}

/// A reference to a value that can only be safely accessed on the main
/// thread. This provides Send + Sync while ensuring the actual value is
/// only touched on the main thread through the dispatcher.
#[derive(Debug)]
pub struct MainThreadRef<T> {
  id: u64,
  dispatcher: Dispatcher,
  _phantom: PhantomData<T>,
}

impl<T> Clone for MainThreadRef<T> {
  fn clone(&self) -> Self {
    Self {
      id: self.id,
      dispatcher: self.dispatcher.clone(),
      _phantom: PhantomData,
    }
  }
}

// SAFETY: The value is only accessed on the main thread.
unsafe impl<T> Send for MainThreadRef<T> {}
unsafe impl<T> Sync for MainThreadRef<T> {}

impl<T> MainThreadRef<T>
where
  T: 'static,
{
  /// Create a new `MainThreadRef` with an initial value.
  /// This should only be called from the main thread.
  pub fn new(dispatcher: Dispatcher, value: T) -> Self {
    let storage_id = Self::storage_id();

    MAIN_THREAD_STORAGE.with(|storage| {
      storage.lock().unwrap().insert(storage_id, Box::new(value));
    });

    Self {
      id: storage_id,
      dispatcher,
      _phantom: PhantomData,
    }
  }

  fn storage_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
  }

  /// Execute a closure with access to the value on the main thread.
  /// The closure receives an Option<&mut T> - None if the value hasn't
  /// been set.
  pub fn with_mut<F, R>(&self, f: F) -> crate::Result<R>
  where
    F: FnOnce(Option<&mut T>) -> R + Send + 'static,
    R: Send + 'static,
  {
    let id = self.id;
    self.dispatcher.dispatch_sync(move || {
      MAIN_THREAD_STORAGE.with(|storage| {
        let mut storage = storage.lock().unwrap();
        f(storage
          .get_mut(&id)
          .map(|value| value.downcast_mut::<T>().unwrap()))
      })
    })
  }

  /// Execute a closure with immutable access to the value on the main
  /// thread.
  pub fn with<F, R>(&self, f: F) -> crate::Result<R>
  where
    F: FnOnce(&T) -> R + Send + 'static,
    R: Send + 'static,
    T: 'static,
  {
    let id = self.id;
    self.dispatcher.dispatch_sync(move || {
      MAIN_THREAD_STORAGE.with(|storage| {
        let storage = storage.lock().unwrap();

        // TODO: Improve error handling.
        f(storage
          .get(&id)
          .expect("Value not found.")
          .downcast_ref::<T>()
          .unwrap())
      })
    })
  }

  /// Set the value (can only be done from main thread via dispatcher).
  pub fn set(&self, new_value: T) {
    MAIN_THREAD_STORAGE.with(|storage| {
      let mut storage = storage.lock().unwrap();
      storage.insert(self.id, Box::new(new_value));
    });
  }

  /// Check if the value is set (non-blocking, but may be stale).
  pub fn is_set(&self) -> bool {
    MAIN_THREAD_STORAGE.with(|storage| {
      let storage = storage.lock().unwrap();
      storage.contains_key(&self.id)
    })
  }
}
