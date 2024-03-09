use std::cell::{Ref, RefMut};

use super::ContainerRef;

// TODO: Consider renaming to `TilingContainer`.
pub trait CommonContainer {
  fn borrow_parent(&self) -> Ref<'_, Option<ContainerRef>>;
  fn borrow_parent_mut(&self) -> RefMut<'_, Option<ContainerRef>>;
  fn borrow_children(&self) -> Ref<'_, Vec<ContainerRef>>;
  fn borrow_children_mut(&self) -> RefMut<'_, Vec<ContainerRef>>;

  /// Returns a reference to the parent node, unless this node is the root
  /// of the tree.
  ///
  /// # Panics
  ///
  /// Panics if the node is currently mutability borrowed.
  fn parent(&self) -> Option<ContainerRef> {
    self.borrow_parent().clone()
  }

  fn insert_child(&self, target_index: usize, child: ContainerRef) {
    self
      .borrow_children_mut()
      .insert(target_index, child.clone());

    *child.common().borrow_parent_mut() = Some(child.clone());
  }

  fn grandparent(&self) -> Option<ContainerRef> {
    self.parent()?.common().parent()
  }
}
