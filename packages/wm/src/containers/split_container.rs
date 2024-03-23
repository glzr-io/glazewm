use std::{
  cell::{Ref, RefCell, RefMut},
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::TilingDirection, impl_common_behavior,
  impl_position_behavior_as_resizable, impl_tiling_behavior,
};

use super::{
  traits::{CommonBehavior, PositionBehavior, TilingBehavior},
  ContainerType, TilingContainer,
};

#[derive(Clone, Debug)]
pub struct SplitContainer(Rc<RefCell<SplitContainerInner>>);

#[derive(Debug)]
struct SplitContainerInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  children: Vec<TilingContainer>,
  size_percent: f32,
  tiling_direction: TilingDirection,
}

impl SplitContainer {
  pub fn new(tiling_direction: TilingDirection) -> Self {
    let split = SplitContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: Vec::new(),
      size_percent: 1.0,
      tiling_direction,
    };

    Self(Rc::new(RefCell::new(split)))
  }
}

impl_common_behavior!(SplitContainer, ContainerType::Split);
impl_tiling_behavior!(SplitContainer);
impl_position_behavior_as_resizable!(SplitContainer);
