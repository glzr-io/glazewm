use enum_dispatch::enum_dispatch;

use crate::{
  common::Rect,
  containers::{
    Container, DirectionContainer, TilingContainer, WindowContainer,
  },
};

#[enum_dispatch]
pub trait PositionBehavior {
  fn width(&self) -> i32;

  fn height(&self) -> i32;

  fn x(&self) -> i32;

  fn y(&self) -> i32;

  fn to_rect(&self) -> Rect {
    Rect::from_xy(self.x(), self.y(), self.width(), self.height())
  }
}

/// Implements the `PositionBehavior` trait for tiling containers that can
/// be resized. Specifically, this is for `SplitContainer` and `TilingWindow`.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with an `id` and a `parent` field.
#[macro_export]
macro_rules! impl_position_behavior_as_resizable {
  ($struct_name:ident) => {
    impl PositionBehavior for $struct_name {
      fn width(&self) -> i32 {
        todo!()
      }

      fn height(&self) -> i32 {
        todo!()
      }

      fn x(&self) -> i32 {
        todo!()
      }

      fn y(&self) -> i32 {
        todo!()
      }
    }
  };
}
