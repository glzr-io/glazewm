use enum_dispatch::enum_dispatch;

use crate::{common::TilingDirection, containers::DirectionContainer};

#[enum_dispatch]
pub trait DirectionBehavior {
  fn tiling_direction(&self) -> TilingDirection;

  fn set_tiling_direction(&self, tiling_direction: TilingDirection);
}

/// Implements the `DirectionBehavior` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with a `tiling_direction` field.
#[macro_export]
macro_rules! impl_direction_behavior {
  ($struct_name:ident) => {
    impl DirectionBehavior for $struct_name {
      fn tiling_direction(&self) -> TilingDirection {
        self.0.borrow().tiling_direction.clone()
      }

      fn set_tiling_direction(&self, tiling_direction: TilingDirection) {
        self.0.borrow_mut().tiling_direction = tiling_direction;
      }
    }
  };
}
