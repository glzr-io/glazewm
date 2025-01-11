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
  containers::{
    traits::{CommonGetters, PositionGetters},
    Container, DirectionContainer, TilingContainer, WindowContainer,
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
      .map(|child| child.to_dto())
      .try_collect()?;

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
      hardware_id: self.native().hardware_id()?.cloned(),
      working_rect: self.native().working_rect()?.clone(),
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
