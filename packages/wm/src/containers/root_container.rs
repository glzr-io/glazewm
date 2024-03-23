use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::{impl_common_behavior, impl_tiling_behavior};

use super::{
  traits::{CommonBehavior, TilingBehavior},
  ContainerType, TilingContainer,
};

/// Root node of the container tree.
#[derive(Clone, Debug)]
pub struct RootContainer(Rc<RefCell<RootContainerInner>>);

#[derive(Debug)]
struct RootContainerInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  children: Vec<TilingContainer>,
  size_percent: f32,
}

impl RootContainer {
  pub fn new() -> Self {
    let root = RootContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: Vec::new(),
      size_percent: 1.0,
    };

    Self(Rc::new(RefCell::new(root)))
  }
}

impl_common_behavior!(RootContainer, ContainerType::Root);
impl_tiling_behavior!(RootContainer);
