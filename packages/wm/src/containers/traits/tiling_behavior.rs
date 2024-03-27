use enum_dispatch::enum_dispatch;

use crate::containers::{Container, TilingContainer};

use super::CommonBehavior;

#[enum_dispatch]
pub trait TilingBehavior: CommonBehavior {
  fn as_tiling_container(&self) -> TilingContainer;

  fn insert_child(&self, target_index: usize, child: Container) {
    self
      .borrow_children_mut()
      .insert(target_index, child.clone());

    *child.borrow_parent_mut() = Some(self.as_tiling_container());
  }
}

/// Implements the `TilingBehavior` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with a `children` field.
#[macro_export]
macro_rules! impl_tiling_behavior {
  ($struct_name:ident) => {
    impl TilingBehavior for $struct_name {
      fn as_tiling_container(&self) -> TilingContainer {
        self.clone().into()
      }
    }
  };
}
