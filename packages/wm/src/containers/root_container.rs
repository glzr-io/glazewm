use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::bail;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
  traits::{CommonGetters, PositionGetters},
  Container, ContainerDto, DirectionContainer, TilingContainer,
  WindowContainer,
};
use crate::{common::Rect, impl_common_getters, impl_container_debug};

/// Root node of the container tree.
#[derive(Clone)]
pub struct RootContainer(Rc<RefCell<RootContainerInner>>);

struct RootContainerInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
}

/// User-friendly representation of a root container.
///
/// Used for IPC and debug logging.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RootContainerDto {
  id: Uuid,
  parent: Option<Uuid>,
  children: Vec<ContainerDto>,
  child_focus_order: Vec<Uuid>,
}

impl RootContainer {
  pub fn new() -> Self {
    let root = RootContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
    };

    Self(Rc::new(RefCell::new(root)))
  }

  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    let children = self
      .children()
      .iter()
      .map(|child| child.to_dto())
      .try_collect()?;

    Ok(ContainerDto::Root(RootContainerDto {
      id: self.id(),
      parent: None,
      children,
      child_focus_order: self.0.borrow().child_focus_order.clone().into(),
    }))
  }
}

impl_container_debug!(RootContainer);
impl_common_getters!(RootContainer);

impl PositionGetters for RootContainer {
  fn to_rect(&self) -> anyhow::Result<Rect> {
    bail!("Root container does not have a position.")
  }
}
