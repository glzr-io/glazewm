use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  fmt,
  rc::Rc,
};

use anyhow::{bail, Context};
use uuid::Uuid;

use crate::{
  common::{LengthValue, TilingDirection},
  impl_common_getters, impl_direction_getters,
  impl_position_getters_as_resizable, impl_tiling_getters,
  windows::{NonTilingWindowDto, TilingWindowDto},
};

use super::{
  traits::{
    CommonGetters, DirectionGetters, PositionGetters, TilingGetters,
  },
  Container, ContainerType, DirectionContainer, TilingContainer,
  WindowContainer,
};

#[derive(Clone)]
pub struct SplitContainer(Rc<RefCell<SplitContainerInner>>);

struct SplitContainerInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  size_percent: f32,
  tiling_direction: TilingDirection,
  inner_gap: LengthValue,
}

impl SplitContainer {
  pub fn new(
    tiling_direction: TilingDirection,
    inner_gap: LengthValue,
  ) -> Self {
    let split = SplitContainerInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      size_percent: 1.0,
      tiling_direction,
      inner_gap,
    };

    Self(Rc::new(RefCell::new(split)))
  }

  pub fn inner_gap(&self) -> LengthValue {
    self.0.borrow().inner_gap.clone()
  }

  pub fn to_dto(&self) -> anyhow::Result<SplitContainerDto> {
    let children = self
      .children()
      .iter()
      .map(|child| match child {
        Container::NonTilingWindow(c) => {
          Ok(SplitContainerChildDto::NonTilingWindow(c.to_dto()?))
        }
        Container::TilingWindow(c) => {
          Ok(SplitContainerChildDto::TilingWindow(c.to_dto()?))
        }
        Container::Split(c) => {
          Ok(SplitContainerChildDto::Split(c.to_dto()?))
        }
        _ => bail!("Split container has an invalid child type."),
      })
      .try_collect()?;

    Ok(SplitContainerDto {
      id: self.id(),
      parent: self.parent().map(|p| p.id()),
      children,
      child_focus_order: self.0.borrow().child_focus_order.clone().into(),
      size_percent: self.size_percent(),
      width: self.width()?,
      height: self.height()?,
      x: self.x()?,
      y: self.y()?,
      tiling_direction: self.tiling_direction(),
    })
  }
}

impl_common_getters!(SplitContainer, ContainerType::Split);
impl_tiling_getters!(SplitContainer);
impl_direction_getters!(SplitContainer);
impl_position_getters_as_resizable!(SplitContainer);

impl fmt::Debug for SplitContainer {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(&self.to_dto().map_err(|_| std::fmt::Error), f)
  }
}

/// User-friendly representation of a split container.
///
/// Used for IPC and debug logging.
#[derive(Debug)]
pub struct SplitContainerDto {
  id: Uuid,
  parent: Option<Uuid>,
  children: Vec<SplitContainerChildDto>,
  child_focus_order: Vec<Uuid>,
  size_percent: f32,
  width: i32,
  height: i32,
  x: i32,
  y: i32,
  tiling_direction: TilingDirection,
}

#[derive(Debug)]
pub enum SplitContainerChildDto {
  NonTilingWindow(NonTilingWindowDto),
  TilingWindow(TilingWindowDto),
  Split(SplitContainerDto),
}
