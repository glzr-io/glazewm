use std::sync::{Arc, Mutex};

/// A thread-safe type that provides lazy initialization and caching of a
/// value.
#[derive(Debug, Clone)]
pub struct Memo<T>
where
  T: Clone,
{
  value: Arc<Mutex<Option<T>>>,
}

impl<T> Default for Memo<T>
where
  T: Clone,
{
  fn default() -> Self {
    Self {
      value: Arc::new(Mutex::new(None)),
    }
  }
}

impl<T> Memo<T>
where
  T: Clone,
{
  /// Creates a new `Memo` instance with `None` as the initial value.
  pub fn new() -> Self {
    Self::default()
  }

  /// Retrieves the cached value if it exists, otherwise initializes it
  /// using the provided closure.
  pub fn get_or_init<F, R>(
    &self,
    retriever_fn: F,
    arg: &R,
  ) -> anyhow::Result<T>
  where
    F: FnOnce(&R) -> anyhow::Result<T>,
    T: Clone,
  {
    let mut value_ref = self.value.lock().unwrap();

    value_ref
      .as_ref()
      .map(|value| Ok(value.clone()))
      .unwrap_or_else(|| {
        let value = retriever_fn(arg)?;
        *value_ref = Some(value.clone());
        Ok(value)
      })
  }

  /// Refreshes the cached value by generating a new value using the
  /// provided closure.
  pub fn update<F, R>(&self, retriever_fn: F, arg: &R) -> anyhow::Result<T>
  where
    F: FnOnce(&R) -> anyhow::Result<T>,
    T: Clone,
  {
    let mut value_ref = self.value.lock().unwrap();

    let value = retriever_fn(arg)?;
    *value_ref = Some(value.clone());
    Ok(value)
  }
}
