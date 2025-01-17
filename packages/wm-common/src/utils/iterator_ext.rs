use std::{collections::HashSet, hash::Hash};

// Extension trait that adds unique filtering capability to iterators
pub trait UniqueExt: Iterator {
  /// Returns an iterator that yields unique elements based on the key
  /// function
  ///
  /// The key function must return a value that implements Hash and Eq
  fn unique_by<K, F>(self, key_fn: F) -> UniqueBy<Self, K, F>
  where
    Self: Sized,
    K: Hash + Eq,
    F: FnMut(&Self::Item) -> K;
}

/// Implementation of unique filtering iterator using a key function
pub struct UniqueBy<I: Iterator, K, F> {
  iter: I,
  key_fn: F,
  seen: HashSet<K>,
}

impl<I, K, F> Iterator for UniqueBy<I, K, F>
where
  I: Iterator,
  K: Hash + Eq,
  F: FnMut(&I::Item) -> K,
{
  type Item = I::Item;

  fn next(&mut self) -> Option<Self::Item> {
    for item in self.iter.by_ref() {
      let key = (self.key_fn)(&item);
      if self.seen.insert(key) {
        return Some(item);
      }
    }

    None
  }
}

// Implement the extension trait for all iterators
impl<I: Iterator> UniqueExt for I {
  fn unique_by<K, F>(self, key_fn: F) -> UniqueBy<Self, K, F>
  where
    Self: Sized,
    K: Hash + Eq,
    F: FnMut(&Self::Item) -> K,
  {
    UniqueBy {
      iter: self,
      key_fn,
      seen: HashSet::new(),
    }
  }
}
