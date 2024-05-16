use enum_dispatch::enum_dispatch;

use crate::{
  common::{Direction, TilingDirection},
  containers::{DirectionContainer, TilingContainer},
  windows::TilingWindow,
};

use super::CommonGetters;

#[enum_dispatch]
pub trait TilingDirectionGetters: CommonGetters {
  fn tiling_direction(&self) -> TilingDirection;

  fn set_tiling_direction(&self, tiling_direction: TilingDirection);

  /// Traverses down a container in search of a descendant in the given
  /// direction. For example, for `Direction::Right`, get the right-most
  /// container.
  ///
  /// Any non-tiling containers are ignored.
  fn descendant_in_direction(
    &self,
    direction: &Direction,
  ) -> Option<TilingWindow> {
    let child = self.child_in_direction(&direction)?;

    // Traverse further down if the child is a split container.
    match child {
      TilingContainer::Split(split_child) => {
        split_child.descendant_in_direction(direction)
      }
      TilingContainer::TilingWindow(window) => Some(window),
    }
  }

  fn child_in_direction(
    &self,
    direction: &Direction,
  ) -> Option<TilingContainer> {
    // When the tiling direction is the inverse of the direction, return
    // the last focused tiling child.
    if self.tiling_direction()
      != TilingDirection::from_direction(direction)
    {
      return self
        .child_focus_order()
        .find_map(|c| c.as_tiling_container().ok());
    }

    match direction {
      Direction::Up | Direction::Left => self.tiling_children().next(),
      _ => self.tiling_children().last(),
    }
  }
}

/// Implements the `TilingDirectionGetters` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with a `tiling_direction` field.
#[macro_export]
macro_rules! impl_tiling_direction_getters {
  ($struct_name:ident) => {
    impl TilingDirectionGetters for $struct_name {
      fn tiling_direction(&self) -> TilingDirection {
        self.0.borrow().tiling_direction.clone()
      }

      fn set_tiling_direction(&self, tiling_direction: TilingDirection) {
        self.0.borrow_mut().tiling_direction = tiling_direction;
      }
    }
  };
}
