use enum_dispatch::enum_dispatch;

use crate::containers::{DirectionContainer, TilingContainer};

use super::CommonBehavior;

#[enum_dispatch]
pub trait TilingBehavior: CommonBehavior {
  fn as_tiling_container(&self) -> TilingContainer;

  fn size_percent(&self) -> f32;

  fn set_size_percent(&self, size_percent: f32) -> ();
}

/// Implements the `TilingBehavior` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with a `size_percent` field.
#[macro_export]
macro_rules! impl_tiling_behavior {
  ($struct_name:ident) => {
    impl TilingBehavior for $struct_name {
      fn as_tiling_container(&self) -> TilingContainer {
        self.clone().into()
      }

      fn size_percent(&self) -> f32 {
        self.0.borrow().size_percent
      }

      fn set_size_percent(&self, size_percent: f32) -> () {
        self.0.borrow_mut().size_percent = size_percent;
      }
    }
  };
}
