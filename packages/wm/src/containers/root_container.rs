use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use super::{
  traits::{CommonContainer, TilingContainer},
  ContainerRef, ContainerType,
};

#[derive(Clone, Debug)]
pub struct RootContainerRef(Rc<RefCell<RootContainer>>);

#[derive(Debug)]
pub struct RootContainer {
  id: Uuid,
  pub parent: Option<ContainerRef>,
  pub children: Vec<ContainerRef>,
}

impl RootContainerRef {
  pub fn new() -> Self {
    let root = RootContainer {
      id: Uuid::new_v4(),
      parent: None,
      children: Vec::new(),
    };

    Self(Rc::new(RefCell::new(root)))
  }
}

impl CommonContainer for RootContainerRef {
  fn id(&self) -> Uuid {
    self.0.borrow().id
  }

  fn r#type(&self) -> ContainerType {
    ContainerType::RootContainer
  }
}

impl TilingContainer for RootContainerRef {
  fn borrow_parent(&self) -> Ref<'_, Option<ContainerRef>> {
    Ref::map(self.0.borrow(), |c| &c.parent)
  }

  fn borrow_parent_mut(&self) -> RefMut<'_, Option<ContainerRef>> {
    RefMut::map(self.0.borrow_mut(), |c| &mut c.parent)
  }

  fn borrow_children(&self) -> Ref<'_, Vec<ContainerRef>> {
    Ref::map(self.0.borrow(), |c| &c.children)
  }

  fn borrow_children_mut(&self) -> RefMut<'_, Vec<ContainerRef>> {
    RefMut::map(self.0.borrow_mut(), |c| &mut c.children)
  }
}
