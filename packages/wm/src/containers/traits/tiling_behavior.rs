use std::cell::{Ref, RefMut};

use enum_dispatch::enum_dispatch;

use crate::containers::{Container, TilingContainer};

use super::CommonBehavior;

#[enum_dispatch]
pub trait TilingBehavior: CommonBehavior {
  fn borrow_children(&self) -> Ref<'_, Vec<Container>>;

  fn borrow_children_mut(&self) -> RefMut<'_, Vec<Container>>;

  fn self_as_tiling(&self) -> TilingContainer;

  fn children(&self) -> Vec<Container> {
    self.borrow_children().clone()
  }

  fn insert_child(&self, target_index: usize, child: Container) {
    self
      .borrow_children_mut()
      .insert(target_index, child.clone());

    *child.borrow_parent_mut() = Some(self.self_as_tiling());
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
      fn borrow_children(&self) -> Ref<'_, Vec<Container>> {
        Ref::map(self.0.borrow(), |c| &c.children)
      }

      fn borrow_children_mut(&self) -> RefMut<'_, Vec<Container>> {
        RefMut::map(self.0.borrow_mut(), |c| &mut c.children)
      }

      fn self_as_tiling(&self) -> TilingContainer {
        self.clone().into()
      }
    }
  };
}
