use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
  common::{platform::NativeMonitor, Rect},
  containers::{
    traits::{CommonGetters, PositionGetters},
    Container, ContainerDto, DirectionContainer, TilingContainer,
    WindowContainer,
  },
  impl_common_getters, impl_container_debug,
  workspaces::Workspace,
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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorDto {
  id: Uuid,
  parent: Option<Uuid>,
  children: Vec<ContainerDto>,
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

  pub fn displayed_workspace(&self) -> Option<Workspace> {
    self
      .child_focus_order()
      .next()
      .and_then(|child| child.as_workspace().cloned())
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

  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    let rect = self.to_rect()?;
    let children = self
      .children()
      .iter()
      .map(|child| child.to_dto())
      .try_collect()?;

    Ok(ContainerDto::Monitor(MonitorDto {
      id: self.id(),
      parent: self.parent().map(|parent| parent.id()),
      children,
      child_focus_order: self.0.borrow().child_focus_order.clone().into(),
      width: rect.width(),
      height: rect.height(),
      x: rect.x(),
      y: rect.y(),
      dpi: self.native().dpi()?,
    }))
  }
}

impl_container_debug!(Monitor);
impl_common_getters!(Monitor);

impl PositionGetters for Monitor {
  fn to_rect(&self) -> anyhow::Result<Rect> {
    self.0.borrow().native.rect().cloned()
  }
}
