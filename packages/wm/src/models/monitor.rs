use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{ContainerDto, MonitorDto, Rect};
use wm_platform::NativeMonitor;

use crate::{
  impl_common_getters, impl_container_debug,
  models::{
    Container, DirectionContainer, TilingContainer, WindowContainer,
    Workspace,
  },
  traits::{CommonGetters, PositionGetters},
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

  pub fn displayed_workspace(&self) -> Option<Workspace> {
    self
      .child_focus_order()
      .next()
      .and_then(|child| child.as_workspace().cloned())
  }

  pub fn workspaces(&self) -> Vec<Workspace> {
    self
      .children()
      .into_iter()
      .filter_map(|container| container.as_workspace().cloned())
      .collect()
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

    Ok(dpi != other_dpi)
  }

  pub fn to_dto(&self) -> anyhow::Result<ContainerDto> {
    let rect = self.to_rect()?;
    let children = self
      .children()
      .iter()
      .map(CommonGetters::to_dto)
      .try_collect()?;

    let hardware_id = self.native().hardware_id()?.cloned();
    let machine_id = self.native().machine_id()?;

    Ok(ContainerDto::Monitor(MonitorDto {
      id: self.id(),
      parent_id: self.parent().map(|parent| parent.id()),
      children,
      child_focus_order: self.0.borrow().child_focus_order.clone().into(),
      has_focus: self.has_focus(None),
      width: rect.width(),
      height: rect.height(),
      x: rect.x(),
      y: rect.y(),
      dpi: self.native().dpi()?,
      scale_factor: self.native().scale_factor()?,
      handle: self.native().handle,
      device_name: self.native().device_name()?.clone(),
      device_path: self.native().device_path()?.cloned(),
      hardware_id,
      machine_id, // Clean machine ID extracted from device path
      working_rect: self.native().working_rect()?.clone(),
      index: self.index(),
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

impl std::fmt::Display for Monitor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let native = self.native();
    let device_name = native
      .device_name()
      .map_or_else(|_| "Unknown".to_string(), String::to_string);
    let device_path = native.device_path().unwrap_or_default();
    let hardware_id = native.hardware_id().unwrap_or_default();

    write!(
      f,
      "Monitor(handle={}, device_name={}, device_path={:?}, hardware_id={:?})",
      native.handle, device_name, device_path, hardware_id,
    )
  }
}
