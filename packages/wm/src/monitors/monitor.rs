use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  fmt,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;

use crate::{
  common::platform::NativeMonitor,
  containers::{
    traits::{CommonGetters, PositionGetters},
    Container, ContainerType, DirectionContainer, TilingContainer,
    WindowContainer,
  },
  impl_common_getters,
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
      .borrow_children()
      .front()
      .and_then(|c| c.as_workspace())
      .cloned()
  }

  /// Whether there is a difference in DPI between this monitor and the
  /// parent monitor of another container.
  pub fn has_dpi_difference(
    &self,
    other: &Container,
  ) -> anyhow::Result<bool> {
    let other_monitor = other.monitor().context("No parent monitor.")?;

    Ok(self.native().dpi()? != other_monitor.native().dpi()?)
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

impl_common_getters!(Monitor, ContainerType::Monitor);

impl PositionGetters for Monitor {
  fn width(&self) -> anyhow::Result<i32> {
    Ok(self.0.borrow().native.width()?)
  }

  fn height(&self) -> anyhow::Result<i32> {
    Ok(self.0.borrow().native.height()?)
  }

  fn x(&self) -> anyhow::Result<i32> {
    Ok(self.0.borrow().native.x()?)
  }

  fn y(&self) -> anyhow::Result<i32> {
    Ok(self.0.borrow().native.y()?)
  }
}

impl fmt::Debug for Monitor {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(&self.to_dto().map_err(|_| std::fmt::Error), f)
  }
}

/// User-friendly representation of a monitor.
///
/// Used for IPC and debug logging.
#[derive(Debug)]
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
