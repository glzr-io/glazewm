use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::Rc,
};

use anyhow::Context;
use uuid::Uuid;
use wm_common::{ContainerDto, GapsConfig, MonitorDto};
use wm_platform::{Display, Rect, RectDelta};

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

  pub fn set_native_properties(
    &self,
    native_properties: NativeMonitorProperties,
  ) {
    self.0.borrow_mut().native_properties = native_properties;
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

  /// Bounds matching `Workspace::max_workspace_rect` for this monitor when
  /// using the given gap configuration.
  ///
  /// Used when positioning a window that is about to be released to the OS on
  /// a monitor that has no WM workspace (for example when
  /// `multi_monitor_workspaces` is disabled).
  pub fn max_workspace_rect_from_gaps(
    &self,
    gaps_config: &GapsConfig,
  ) -> Rect {
    let multi_window_rect = self.rect_with_outer_gap(
      &gaps_config.outer_gap,
      gaps_config,
    );

    let Some(single_gap) = &gaps_config.single_window_outer_gap else {
      return multi_window_rect;
    };

    let single_window_rect =
      self.rect_with_outer_gap(single_gap, gaps_config);

    multi_window_rect.union(&single_window_rect)
  }

  /// Workspace client area for this monitor with a specific outer gap delta.
  fn rect_with_outer_gap(
    &self,
    outer_gap: &RectDelta,
    gaps_config: &GapsConfig,
  ) -> Rect {
    let scale_factor = if gaps_config.scale_with_dpi {
      self.native_properties().scale_factor
    } else {
      1.
    };

    let monitor_bounds = self.native_properties().bounds;
    let working_area_delta = self
      .native_properties()
      .working_area
      .delta(&monitor_bounds);

    monitor_bounds
      .apply_delta(&outer_gap.inverse(), Some(scale_factor))
      .apply_delta(&working_area_delta, None)
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
      #[cfg(target_os = "windows")]
      handle: Some(self.native_properties().handle),
      #[cfg(not(target_os = "windows"))]
      handle: None,
      device_name: self.native_properties().device_name,
      #[cfg(target_os = "windows")]
      device_path: self.native_properties().device_path,
      #[cfg(not(target_os = "windows"))]
      device_path: None,
      #[cfg(target_os = "windows")]
      hardware_id: self.native_properties().hardware_id,
      #[cfg(target_os = "macos")]
      hardware_id: Some(self.native_properties().device_uuid.clone()),
      #[cfg(all(
        not(target_os = "windows"),
        not(target_os = "macos")
      ))]
      hardware_id: None,
      working_rect: self.native_properties().working_area,
      // Defaults to `false`; populated via `set_is_primary_on_dto` at
      // call sites that have access to `UserConfig`.
      is_primary: false,
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
    write!(
      f,
      "Monitor(device_name={})",
      self.native_properties().device_name,
    )
  }
}
