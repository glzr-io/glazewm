use enum_dispatch::enum_dispatch;

use crate::{common::LengthValue, containers::TilingContainer};

use super::CommonGetters;

#[enum_dispatch]
pub trait TilingSizeGetters: CommonGetters {
  fn tiling_size(&self) -> f32;

  fn set_tiling_size(&self, tiling_size: f32) -> ();

  fn inner_gap(&self) -> LengthValue;

  fn set_inner_gap(&self, inner_gap: LengthValue);
}

/// Implements the `TilingSizeGetters` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with a `tiling_size` field.
#[macro_export]
macro_rules! impl_tiling_size_getters {
  ($struct_name:ident) => {
    impl TilingSizeGetters for $struct_name {
      fn tiling_size(&self) -> f32 {
        self.0.borrow().tiling_size
      }

      fn set_tiling_size(&self, tiling_size: f32) -> () {
        self.0.borrow_mut().tiling_size = tiling_size;
      }

      fn inner_gap(&self) -> LengthValue {
        self.0.borrow().inner_gap.clone()
      }

      fn set_inner_gap(&self, inner_gap: LengthValue) -> () {
        self.0.borrow_mut().inner_gap = inner_gap;
      }
    }
  };
}