use std::collections::VecDeque;

pub trait VecDequeExt<T>
where
  T: PartialEq,
{
  /// Shifts a value to a specified index in a `VecDeque`.
  ///
  /// Inserts at index if value doesn't already exist in the `VecDeque`.
  fn shift_to_index(&mut self, target_index: usize, item: T);

  /// Replaces the first occurrence of a value with a new value in a
  /// `VecDeque`.
  fn replace(&mut self, old_value: &T, new_value: T);
}

impl<T> VecDequeExt<T> for VecDeque<T>
where
  T: PartialEq,
{
  fn shift_to_index(&mut self, target_index: usize, value: T) {
    if let Some(index) = self.iter().position(|e| e == &value) {
      self.remove(index);
      self.insert(target_index, value);
    }
  }

  fn replace(&mut self, old_value: &T, new_value: T) {
    if let Some(index) = self.iter().position(|e| e == old_value) {
      self[index] = new_value;
    }
  }
}
