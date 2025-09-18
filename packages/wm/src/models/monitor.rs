use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{ContainerDto, MonitorDto};
use wm_platform::{Display, Rect};

use crate::{
  impl_common_getters, impl_container_debug,
  models::{
    Container, DirectionContainer, NativeMonitorProperties,
    TilingContainer, WindowContainer, Workspace,
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
  native: Display,
  native_properties: NativeMonitorProperties,
}

impl Monitor {
  pub fn new(
    native_display: Display,
    native_properties: NativeMonitorProperties,
  ) -> Self {
    let monitor = MonitorInner {
      id: Uuid::new_v4(),
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
      native: native_display,
      native_properties,
    };

    Self(Rc::new(RefCell::new(monitor)))
  }

  pub fn native(&self) -> Display {
    self.0.borrow().native.clone()
  }

  pub fn set_native(&self, native: Display) {
    self.0.borrow_mut().native = native;
  }

  pub fn native_properties(&self) -> NativeMonitorProperties {
    self.0.borrow().native_properties.clone()
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
    let dpi = self.native_properties().dpi;

    let other_dpi = other
      .monitor()
      .map(|monitor| monitor.native_properties().dpi)
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
      dpi: self.native_properties().dpi,
      scale_factor: self.native_properties().scale_factor,
      // handle: self.native().id().0,
      // device_name: self.native_properties().device_name,
      // device_path: self.native_properties().device_path,
      // hardware_id: self.native_properties().hardware_id,
      working_rect: self.native_properties().working_area.clone(),
    }))
  }
}

impl_container_debug!(Monitor);
impl_common_getters!(Monitor);

impl PositionGetters for Monitor {
  fn to_rect(&self) -> anyhow::Result<Rect> {
    Ok(self.0.borrow().native_properties.bounds.clone())
  }
}

impl std::fmt::Display for Monitor {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let native = self.native();
    // let device_name = native
    //   .device_name()
    //   .map_or_else(|_| "Unknown".to_string(), String::to_string);
    // let device_path = native.device_path().unwrap_or_default();
    // let hardware_id = native.hardware_id().unwrap_or_default();

    // write!(
    //   f,
    //   "Monitor(handle={}, device_name={}, device_path={:?},
    // hardware_id={:?})",   native.id(), device_name, device_path,
    // hardware_id, )
    write!(f, "Monitor(handle={:?})", native.id(),)
  }
}
