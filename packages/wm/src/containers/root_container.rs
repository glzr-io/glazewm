use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::impl_common_behavior;

use super::{
  traits::{CommonBehavior, TilingBehavior},
  ContainerType, TilingContainer,
};

#[derive(Clone, Debug)]
pub struct RootContainer(Rc<RefCell<RootContainerInner>>);

#[derive(Debug)]
struct RootContainerInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  children: Vec<TilingContainer>,
}

impl RootContainer {
  pub fn new() -> Self {
    let root = RootContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: Vec::new(),
    };

    Self(Rc::new(RefCell::new(root)))
  }
}

impl_common_behavior!(RootContainer, ContainerType::Root);

impl TilingBehavior for RootContainer {
  fn borrow_tiling_children(&self) -> Ref<'_, Vec<TilingContainer>> {
    Ref::map(self.0.borrow(), |c| &c.children)
  }

  fn borrow_tiling_children_mut(
    &self,
  ) -> RefMut<'_, Vec<TilingContainer>> {
    RefMut::map(self.0.borrow_mut(), |c| &mut c.children)
  }
}
