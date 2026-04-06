//! Test utilities for creating mock container instances.
//!
//! This module provides default values and helper functions used by the
//! mock builders in the model modules.

#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod mocks {
  use wm_common::{
    GapsConfig, TilingDirection, WindowState, WorkspaceConfig,
  };
  use wm_platform::{Display, NativeWindow, Rect, RectDelta};

  use crate::{
    commands::container::attach_container,
    models::{
      Container, Monitor, NativeMonitorProperties, NativeWindowProperties,
      NonTilingWindow, SplitContainer, TilingContainer, TilingWindow,
      Workspace,
    },
    traits::{CommonGetters, TilingSizeGetters},
  };

  /// Default monitor dimensions (1680x1050).
  pub const DEFAULT_MONITOR_WIDTH: i32 = 1680;
  pub const DEFAULT_MONITOR_HEIGHT: i32 = 1050;

  /// Taskbar height offset for working area.
  pub const TASKBAR_HEIGHT: i32 = 50;

  /// Default DPI for monitors.
  pub const DEFAULT_DPI: u32 = 96;

  /// Default scale factor.
  pub const DEFAULT_SCALE_FACTOR: f32 = 1.0;

  /// Default window dimensions (300x200).
  pub const DEFAULT_WINDOW_WIDTH: i32 = 300;
  pub const DEFAULT_WINDOW_HEIGHT: i32 = 200;

  /// Default monitor bounds (0,0 to 1680x1050).
  pub fn default_bounds() -> Rect {
    Rect::from_xy(0, 0, DEFAULT_MONITOR_WIDTH, DEFAULT_MONITOR_HEIGHT)
  }

  /// Default working area (shrunk by taskbar height at bottom).
  pub fn default_working_area() -> Rect {
    Rect::from_xy(
      0,
      0,
      DEFAULT_MONITOR_WIDTH,
      DEFAULT_MONITOR_HEIGHT - TASKBAR_HEIGHT,
    )
  }

  /// Default window rect (300x200 at origin).
  pub fn default_window_rect() -> Rect {
    Rect::from_xy(0, 0, DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
  }

  /// Default gaps configuration (zero gaps).
  pub fn default_gaps_config() -> GapsConfig {
    GapsConfig::default()
  }

  /// Default border delta (zero).
  pub fn default_border_delta() -> RectDelta {
    RectDelta::zero()
  }

  // ===== MONITOR =====

  pub(crate) fn build_mock_monitor(
    device_name: String,
    bounds: Rect,
    working_area: Rect,
    dpi: u32,
    scale_factor: f32,
    native: Display,
    workspaces: Vec<Workspace>,
  ) -> Monitor {
    let properties = NativeMonitorProperties::mock()
      .device_name(device_name)
      .bounds(bounds)
      .working_area(working_area)
      .dpi(dpi)
      .scale_factor(scale_factor)
      .call();

    let monitor = Monitor::new(native, properties);

    for workspace in workspaces {
      attach_container(&workspace.into(), &monitor.clone().into(), None)
        .unwrap();
    }

    monitor
  }

  // ===== WORKSPACE =====

  pub(crate) fn build_mock_workspace(
    name: String,
    display_name: Option<String>,
    tiling_direction: TilingDirection,
    gaps_config: GapsConfig,
    tiling_containers: Vec<TilingContainer>,
    non_tiling_windows: Vec<NonTilingWindow>,
  ) -> Workspace {
    let config = WorkspaceConfig {
      name,
      display_name,
      bind_to_monitor: None,
      keep_alive: false,
    };

    let workspace = Workspace::new(config, gaps_config, tiling_direction);

    for child in tiling_containers {
      attach_container(&child.into(), &workspace.clone().into(), None)
        .unwrap();
    }

    for child in non_tiling_windows {
      attach_container(&child.into(), &workspace.clone().into(), None)
        .unwrap();
    }

    workspace
  }

  // ===== SPLIT CONTAINER =====

  #[allow(clippy::cast_precision_loss)]
  pub(crate) fn build_mock_split_container(
    tiling_direction: TilingDirection,
    gaps_config: GapsConfig,
    distribute_tiling_sizes: bool,
    tiling_containers: Vec<TilingContainer>,
  ) -> SplitContainer {
    let split = SplitContainer::new(tiling_direction, gaps_config);

    let tiling_size =
      if distribute_tiling_sizes && !tiling_containers.is_empty() {
        1.0 / tiling_containers.len() as f32
      } else {
        1.0
      };

    for child in tiling_containers {
      let container: Container = child.into();
      if distribute_tiling_sizes {
        if let Ok(tiling_child) = container.as_tiling_container() {
          tiling_child.set_tiling_size(tiling_size);
        }
      }
      attach_container(&container, &split.clone().into(), None).unwrap();
    }

    split
  }

  // ===== TILING WINDOW =====

  pub(crate) fn build_mock_tiling_window(
    tiling_size: f32,
    title: String,
    process_name: String,
    floating_placement: Rect,
    gaps_config: GapsConfig,
    native: NativeWindow,
  ) -> TilingWindow {
    let properties = NativeWindowProperties::mock()
      .title(title)
      .process_name(process_name)
      .frame(floating_placement.clone())
      .call();

    let window = TilingWindow::new(
      None,
      native,
      properties,
      None,
      default_border_delta(),
      floating_placement,
      false,
      gaps_config,
      vec![],
      None,
    );

    window.set_tiling_size(tiling_size);
    window
  }

  // ===== NON-TILING WINDOW =====

  pub(crate) fn build_mock_non_tiling_window(
    title: String,
    process_name: String,
    floating_placement: Rect,
    state: WindowState,
    native: NativeWindow,
  ) -> NonTilingWindow {
    let properties = NativeWindowProperties::mock()
      .title(title)
      .process_name(process_name)
      .frame(floating_placement.clone())
      .call();

    NonTilingWindow::new(
      None,
      native,
      properties,
      state,
      None,
      default_border_delta(),
      None,
      floating_placement,
      false,
      vec![],
      None,
    )
  }
}
