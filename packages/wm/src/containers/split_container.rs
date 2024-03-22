use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::{common::TilingDirection, impl_common_container};

use super::{
  traits::{CommonContainer, TilingContainer},
  Container, ContainerType,
};

#[derive(Clone, Debug)]
pub struct SplitContainer(Rc<RefCell<SplitContainerInner>>);

#[derive(Debug)]
pub struct SplitContainerInner {
  id: Uuid,
  parent: Option<Container>,
  children: Vec<Container>,
  tiling_direction: TilingDirection,
}

impl SplitContainer {
  pub fn new(tiling_direction: TilingDirection) -> Self {
    let split = SplitContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: Vec::new(),
      tiling_direction,
    };

    Self(Rc::new(RefCell::new(split)))
  }
}

impl_common_container!(SplitContainer, ContainerType::Split);

impl TilingContainer for SplitContainer {
  fn borrow_tiling_children(&self) -> Ref<'_, Vec<Container>> {
    Ref::map(self.0.borrow(), |c| &c.children)
  }

  fn borrow_tiling_children_mut(&self) -> RefMut<'_, Vec<Container>> {
    RefMut::map(self.0.borrow_mut(), |c| &mut c.children)
  }
}
