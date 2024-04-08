use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use uuid::Uuid;

use crate::{
  common::TilingDirection, impl_common_behavior, impl_direction_behavior,
  impl_position_behavior_as_resizable, impl_tiling_behavior,
};

use super::{
  traits::{
    CommonBehavior, DirectionBehavior, PositionBehavior, TilingBehavior,
  },
  Container, ContainerType, TilingContainer,
};

#[derive(Clone, Debug)]
pub struct SplitContainer(Rc<RefCell<SplitContainerInner>>);

#[derive(Debug)]
struct SplitContainerInner {
  id: Uuid,
  parent: Option<TilingContainer>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  size_percent: f32,
  tiling_direction: TilingDirection,
}

impl SplitContainer {
  pub fn new(tiling_direction: TilingDirection) -> Self {
    let split = SplitContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      size_percent: 1.0,
      tiling_direction,
    };

    Self(Rc::new(RefCell::new(split)))
  }
}

impl_common_behavior!(SplitContainer, ContainerType::Split);
impl_tiling_behavior!(SplitContainer);
impl_direction_behavior!(SplitContainer);
impl_position_behavior_as_resizable!(SplitContainer);
