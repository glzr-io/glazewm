use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{
  ContainerDto, GapsConfig, Rect, SplitContainerDto, TilingDirection,
};

use crate::{
  impl_common_getters, impl_container_debug,
  impl_position_getters_as_resizable, impl_tiling_direction_getters,
  impl_tiling_size_getters,
  models::{
    Container, DirectionContainer, TilingContainer, WindowContainer,
  },
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters,
    TilingSizeGetters,
  },
};

#[derive(Clone)]
pub struct SplitContainer(Rc<RefCell<SplitContainerInner>>);

struct SplitContainerInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  tiling_size: f32,
  tiling_direction: TilingDirection,
  gaps_config: GapsConfig,
}

impl SplitContainer {
  pub fn new(
    tiling_direction: TilingDirection,
    gaps_config: GapsConfig,
  ) -> Self {
    let split = SplitContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      tiling_size: 1.0,
      tiling_direction,
      gaps_config,
    };

    Self(Rc::new(RefCell::new(split)))
  }

  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    let rect = self.to_rect()?;
    let children = self
      .children()
      .iter()
      .map(CommonGetters::to_dto)
      .try_collect()?;

    Ok(ContainerDto::Split(SplitContainerDto {
      id: self.id(),
      parent_id: self.parent().map(|parent| parent.id()),
      children,
      child_focus_order: self.0.borrow().child_focus_order.clone().into(),
      has_focus: self.has_focus(None),
      tiling_size: self.tiling_size(),
      tiling_direction: self.tiling_direction(),
      width: rect.width(),
      height: rect.height(),
      x: rect.x(),
      y: rect.y(),
    }))
  }
}

impl_container_debug!(SplitContainer);
impl_common_getters!(SplitContainer);
impl_tiling_size_getters!(SplitContainer);
impl_tiling_direction_getters!(SplitContainer);
impl_position_getters_as_resizable!(SplitContainer);
