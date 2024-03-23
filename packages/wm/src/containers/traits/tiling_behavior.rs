use std::cell::{Ref, RefMut};

use enum_dispatch::enum_dispatch;

use crate::containers::{Container, TilingContainer};

use super::CommonBehavior;

#[enum_dispatch]
pub trait TilingBehavior: CommonBehavior {
  fn borrow_tiling_children(&self) -> Ref<'_, Vec<Container>>;

  fn borrow_tiling_children_mut(&self) -> RefMut<'_, Vec<Container>>;

  fn tiling_children(&self) -> Vec<Container> {
    self.borrow_tiling_children().clone()
  }

  fn insert_tiling_child(&self, target_index: usize, child: Container) {
    self
      .borrow_tiling_children_mut()
      .insert(target_index, child.clone());

    *child.borrow_parent_mut() = Some(child.clone());
  }
}
