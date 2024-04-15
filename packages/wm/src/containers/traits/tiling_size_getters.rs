use enum_dispatch::enum_dispatch;

use crate::containers::TilingContainer;

use super::CommonGetters;

#[enum_dispatch]
pub trait TilingSizeGetters: CommonGetters {
  fn size_percent(&self) -> f32;

  fn set_size_percent(&self, size_percent: f32) -> ();
}

/// Implements the `TilingSizeGetters` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with a `size_percent` field.
#[macro_export]
macro_rules! impl_tiling_size_getters {
  ($struct_name:ident) => {
    impl TilingSizeGetters for $struct_name {
      fn size_percent(&self) -> f32 {
        self.0.borrow().size_percent
      }

      fn set_size_percent(&self, size_percent: f32) -> () {
        self.0.borrow_mut().size_percent = size_percent;
      }
    }
  };
}
