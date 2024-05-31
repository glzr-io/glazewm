use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use serde::Serialize;
use uuid::Uuid;

use crate::{
  common::platform::NativeMonitor,
  containers::{
    traits::{CommonGetters, PositionGetters},
    Container, ContainerType, DirectionContainer, TilingContainer,
    WindowContainer,
  },
  impl_common_getters, impl_container_debug, impl_container_serialize,
  workspaces::{Workspace, WorkspaceDto},
};

#[derive(Clone)]
pub struct Monitor(Rc<RefCell<MonitorInner>>);

struct MonitorInner {
  id: Uuid,
  parent: Option<Container>,
  children: VecDeque<Container>,
  child_focus_order: VecDeque<Uuid>,
  native: NativeMonitor,
}

/// User-friendly representation of a monitor.
///
/// Used for IPC and debug logging.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorDto {
  id: Uuid,
  parent: Option<Uuid>,
  children: Vec<WorkspaceDto>,
  child_focus_order: Vec<Uuid>,
  width: i32,
  height: i32,
  x: i32,
  y: i32,
  dpi: f32,
}

impl Monitor {
  pub fn new(native_monitor: NativeMonitor) -> Self {
    let monitor = MonitorInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      native: native_monitor,
    };

    Self(Rc::new(RefCell::new(monitor)))
  }

  pub fn native(&self) -> NativeMonitor {
    self.0.borrow().native.clone()
  }

  pub fn set_native(&self, native: NativeMonitor) {
    self.0.borrow_mut().native = native;
  }

  pub fn name(&self) -> anyhow::Result<String> {
    self.native().device_name().cloned()
  }

  pub fn displayed_workspace(&self) -> Option<Workspace> {
    self
      .child_focus_order()
      .next()
      .and_then(|c| c.as_workspace().cloned())
  }

  /// Whether there is a difference in DPI between this monitor and the
  /// parent monitor of another container.
  pub fn has_dpi_difference(
    &self,
    other: &Container,
  ) -> anyhow::Result<bool> {
    let dpi = self.native().dpi()?;
    let other_dpi = other
      .monitor()
      .and_then(|monitor| monitor.native().dpi().ok())
      .context("Failed to get DPI of other monitor.")?;

    Ok((dpi - other_dpi).abs() < f32::EPSILON)
  }

  pub fn to_dto(&self) -> anyhow::Result<MonitorDto> {
    let children = self
      .children()
      .iter()
      .map(|c| {
        c.as_workspace()
          .context("Monitor has an invalid child type.")
          .and_then(|c| c.to_dto())
      })
      .try_collect()?;

    Ok(MonitorDto {
      id: self.id(),
      parent: self.parent().map(|p| p.id()),
      children,
      child_focus_order: self.0.borrow().child_focus_order.clone().into(),
      width: self.width()?,
      height: self.height()?,
      x: self.x()?,
      y: self.y()?,
      dpi: self.native().dpi()?,
    })
  }
}

impl_container_debug!(Monitor);
impl_container_serialize!(Monitor);
impl_common_getters!(Monitor, ContainerType::Monitor);

impl PositionGetters for Monitor {
  fn width(&self) -> anyhow::Result<i32> {
    Ok(self.0.borrow().native.working_rect()?.width())
  }

  fn height(&self) -> anyhow::Result<i32> {
    Ok(self.0.borrow().native.working_rect()?.height())
  }

  fn x(&self) -> anyhow::Result<i32> {
    Ok(self.0.borrow().native.working_rect()?.x())
  }

  fn y(&self) -> anyhow::Result<i32> {
    Ok(self.0.borrow().native.working_rect()?.y())
  }
}
