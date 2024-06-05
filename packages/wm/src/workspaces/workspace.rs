use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::{bail, Context};
use serde::Serialize;
use uuid::Uuid;

use crate::{
  common::{RectDelta, TilingDirection},
  containers::{
    traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
    Container, ContainerType, DirectionContainer, SplitContainerDto,
    TilingContainer, WindowContainer,
  },
  impl_common_getters, impl_container_debug, impl_container_serialize,
  impl_tiling_direction_getters,
  user_config::WorkspaceConfig,
  windows::{NonTilingWindowDto, TilingWindowDto},
};

#[derive(Clone)]
pub struct Workspace(Rc<RefCell<WorkspaceInner>>);

#[derive(Debug)]
struct WorkspaceInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  tiling_direction: TilingDirection,
  config: WorkspaceConfig,
  outer_gaps: RectDelta,
}

/// User-friendly representation of a workspace.
///
/// Used for IPC and debug logging.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDto {
  id: Uuid,
  parent: Option<Uuid>,
  children: Vec<WorkspaceChildDto>,
  child_focus_order: Vec<Uuid>,
  width: i32,
  height: i32,
  x: i32,
  y: i32,
  tiling_direction: TilingDirection,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum WorkspaceChildDto {
  NonTilingWindow(NonTilingWindowDto),
  TilingWindow(TilingWindowDto),
  Split(SplitContainerDto),
}

impl Workspace {
  pub fn new(
    config: WorkspaceConfig,
    outer_gaps: RectDelta,
    tiling_direction: TilingDirection,
  ) -> Self {
    let workspace = WorkspaceInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      tiling_direction,
      config,
      outer_gaps,
    };

    Self(Rc::new(RefCell::new(workspace)))
  }

  /// Underlying config for the workspace.
  pub fn config(&self) -> WorkspaceConfig {
    self.0.borrow().config.clone()
  }

  /// Update the underlying config for the workspace.
  pub fn set_config(&self, config: WorkspaceConfig) -> () {
    self.0.borrow_mut().config = config;
  }

  /// Whether the workspace is currently displayed by the parent monitor.
  pub fn is_displayed(&self) -> bool {
    self
      .monitor()
      .and_then(|monitor| monitor.displayed_workspace())
      .map(|workspace| workspace.id() == self.id())
      .unwrap_or(false)
  }

  fn outer_gaps(&self) -> Ref<'_, RectDelta> {
    Ref::map(self.0.borrow(), |c| &c.outer_gaps)
  }

  pub fn to_dto(&self) -> anyhow::Result<WorkspaceDto> {
    let children = self
      .children()
      .iter()
      .map(|child| match child {
        Container::NonTilingWindow(c) => {
          Ok(WorkspaceChildDto::NonTilingWindow(c.to_dto()?))
        }
        Container::TilingWindow(c) => {
          Ok(WorkspaceChildDto::TilingWindow(c.to_dto()?))
        }
        Container::Split(c) => Ok(WorkspaceChildDto::Split(c.to_dto()?)),
        _ => bail!("Workspace has an invalid child type."),
      })
      .try_collect()?;

    Ok(WorkspaceDto {
      id: self.id(),
      parent: self.parent().map(|p| p.id()),
      children,
      child_focus_order: self.0.borrow().child_focus_order.clone().into(),
      width: self.width()?,
      height: self.height()?,
      x: self.x()?,
      y: self.y()?,
      tiling_direction: self.tiling_direction(),
    })
  }
}

impl_container_debug!(Workspace);
impl_container_serialize!(Workspace);
impl_common_getters!(Workspace, ContainerType::Workspace);
impl_tiling_direction_getters!(Workspace);

impl PositionGetters for Workspace {
  fn width(&self) -> anyhow::Result<i32> {
    let parent = self.parent().context("Workspace has no parent.")?;

    Ok(
      parent.width()?
        - self.outer_gaps().left.to_pixels(parent.width()?)
        - self.outer_gaps().right.to_pixels(parent.width()?),
    )
  }

  fn height(&self) -> anyhow::Result<i32> {
    let parent = self.parent().context("Workspace has no parent.")?;

    Ok(
      parent.height()?
        - self.outer_gaps().top.to_pixels(parent.height()?)
        - self.outer_gaps().bottom.to_pixels(parent.height()?),
    )
  }

  fn x(&self) -> anyhow::Result<i32> {
    let parent = self.parent().context("Workspace has no parent.")?;

    Ok(parent.x()? + self.outer_gaps().left.to_pixels(parent.width()?))
  }

  fn y(&self) -> anyhow::Result<i32> {
    let parent = self.parent().context("Workspace has no parent.")?;

    Ok(parent.y()? + self.outer_gaps().top.to_pixels(parent.height()?))
  }
}
