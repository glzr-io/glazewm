//! Test utilities for creating mock container instances.
//!
//! This module provides default values and helper functions used by the
//! mock builders in the model modules.

use bon::bon;
use tokio::sync::mpsc;
use wm_common::{
  FloatingStateConfig, GapsConfig, TilingDirection, WindowState, WmEvent,
  WorkspaceConfig,
};
use wm_platform::{Dispatcher, Display, NativeWindow, Rect, RectDelta};

use crate::{
  commands::container::attach_container,
  models::{
    Monitor, NativeMonitorProperties, NativeWindowProperties,
    NonTilingWindow, SplitContainer, TilingContainer, TilingWindow,
    Workspace,
  },
  traits::TilingSizeGetters,
  wm_state::WmState,
};

pub const MOCK_MONITOR_WIDTH: i32 = 1680;
pub const MOCK_MONITOR_HEIGHT: i32 = 1050;
pub const MOCK_TASKBAR_HEIGHT: i32 = 50;
pub const MOCK_DPI: u32 = 96;
pub const MOCK_SCALE_FACTOR: f32 = 1.0;
pub const MOCK_WINDOW_WIDTH: i32 = 300;
pub const MOCK_WINDOW_HEIGHT: i32 = 200;

pub fn mock_bounds() -> Rect {
  Rect::from_xy(0, 0, MOCK_MONITOR_WIDTH, MOCK_MONITOR_HEIGHT)
}

pub fn mock_working_area() -> Rect {
  Rect::from_xy(
    0,
    0,
    MOCK_MONITOR_WIDTH,
    MOCK_MONITOR_HEIGHT - MOCK_TASKBAR_HEIGHT,
  )
}

pub fn mock_window_rect() -> Rect {
  Rect::from_xy(0, 0, MOCK_WINDOW_WIDTH, MOCK_WINDOW_HEIGHT)
}

pub fn mock_border_delta() -> RectDelta {
  RectDelta::zero()
}

pub fn mock_channel_sender<T>() -> mpsc::UnboundedSender<T> {
  mpsc::unbounded_channel().0
}

#[bon]
impl Monitor {
  #[builder]
  pub fn mock(
    #[builder(default = String::new())] device_name: String,
    #[builder(default = mock_bounds())] bounds: Rect,
    #[builder(default = mock_working_area())] working_area: Rect,
    #[builder(default = MOCK_DPI)] dpi: u32,
    #[builder(default = MOCK_SCALE_FACTOR)] scale_factor: f32,
    #[builder(default = Display::mock())] native: Display,
    #[builder(default = vec![])] workspaces: Vec<Workspace>,
  ) -> Self {
    let properties = NativeMonitorProperties::mock()
      .device_name(device_name)
      .bounds(bounds)
      .working_area(working_area)
      .dpi(dpi)
      .scale_factor(scale_factor)
      .call();

    let monitor = Self::new(native, properties);

    for workspace in workspaces {
      attach_container(&workspace.into(), &monitor.clone().into(), None)
        .unwrap();
    }

    monitor
  }
}

#[bon]
impl NativeMonitorProperties {
  #[builder]
  pub fn mock(
    #[builder(default = String::new())] device_name: String,
    #[builder(default = mock_bounds())] bounds: Rect,
    #[builder(default = mock_working_area())] working_area: Rect,
    #[builder(default = MOCK_DPI)] dpi: u32,
    #[builder(default = MOCK_SCALE_FACTOR)] scale_factor: f32,
  ) -> Self {
    Self {
      device_name,
      bounds,
      working_area,
      dpi,
      scale_factor,
      #[cfg(target_os = "macos")]
      device_uuid: String::new(),
      #[cfg(target_os = "windows")]
      handle: 0,
      #[cfg(target_os = "windows")]
      hardware_id: None,
      #[cfg(target_os = "windows")]
      device_path: None,
    }
  }
}

#[bon]
impl NativeWindowProperties {
  #[builder]
  pub fn mock(
    #[builder(default = String::new())] title: String,
    #[builder(default = String::new())] process_name: String,
    #[builder(default = mock_window_rect())] frame: Rect,
    #[builder(default = false)] is_minimized: bool,
    #[builder(default = false)] is_maximized: bool,
    #[builder(default = true)] is_resizable: bool,
  ) -> Self {
    Self {
      title,
      process_name,
      frame,
      is_minimized,
      is_maximized,
      is_resizable,
      #[cfg(target_os = "windows")]
      class_name: String::new(),
      #[cfg(target_os = "windows")]
      shadow_borders: mock_border_delta(),
    }
  }
}

#[bon]
impl NonTilingWindow {
  #[builder]
  pub fn mock(
    #[builder(default = String::new())] title: String,
    #[builder(default = String::new())] process_name: String,
    #[builder(default = mock_window_rect())] floating_placement: Rect,
    #[builder(default = WindowState::Floating(FloatingStateConfig::default()))]
    state: WindowState,
    #[builder(default = NativeWindow::mock())] native: NativeWindow,
  ) -> Self {
    let properties = NativeWindowProperties::mock()
      .title(title)
      .process_name(process_name)
      .frame(floating_placement.clone())
      .call();

    Self::new(
      None,
      native,
      properties,
      state,
      None,
      mock_border_delta(),
      None,
      floating_placement,
      false,
      vec![],
      None,
    )
  }
}

#[bon]
impl SplitContainer {
  #[builder]
  #[allow(clippy::cast_precision_loss)]
  pub fn mock(
    #[builder(default = TilingDirection::Horizontal)]
    tiling_direction: TilingDirection,
    #[builder(default = GapsConfig::default())] gaps_config: GapsConfig,
    #[builder(default = vec![])] tiling_containers: Vec<TilingContainer>,
  ) -> Self {
    let split = Self::new(tiling_direction, gaps_config);

    for child in tiling_containers {
      attach_container(&child.into(), &split.clone().into(), None)
        .unwrap();
    }

    split
  }
}

#[bon]
impl TilingWindow {
  #[builder]
  pub fn mock(
    #[builder(default = 1.0)] tiling_size: f32,
    #[builder(default = String::new())] title: String,
    #[builder(default = String::new())] process_name: String,
    #[builder(default = mock_window_rect())] floating_placement: Rect,
    #[builder(default = GapsConfig::default())] gaps_config: GapsConfig,
    #[builder(default = NativeWindow::mock())] native: NativeWindow,
  ) -> Self {
    let properties = NativeWindowProperties::mock()
      .title(title)
      .process_name(process_name)
      .frame(floating_placement.clone())
      .call();

    let window = Self::new(
      None,
      native,
      properties,
      None,
      mock_border_delta(),
      floating_placement,
      false,
      gaps_config,
      vec![],
      None,
    );

    window.set_tiling_size(tiling_size);

    window
  }
}

#[bon]
impl WmState {
  #[builder]
  #[allow(clippy::needless_pass_by_value)]
  pub fn mock(
    #[builder(default = Dispatcher::mock())] dispatcher: Dispatcher,
    #[builder(default = mock_channel_sender())]
    event_tx: mpsc::UnboundedSender<WmEvent>,
    #[builder(default = mock_channel_sender())]
    exit_tx: mpsc::UnboundedSender<()>,
    #[builder(default = vec![])] monitors: Vec<Monitor>,
  ) -> Self {
    let state = WmState::new(dispatcher, event_tx, exit_tx);

    for monitor in monitors {
      attach_container(
        &monitor.clone().into(),
        &state.root_container.clone().into(),
        None,
      )
      .unwrap();
    }

    state
  }
}

#[bon]
impl Workspace {
  #[builder]
  pub fn mock(
    #[builder(default = "1".to_string())] name: String,
    display_name: Option<String>,
    #[builder(default = TilingDirection::Horizontal)]
    tiling_direction: TilingDirection,
    #[builder(default = GapsConfig::default())] gaps_config: GapsConfig,
    #[builder(default = vec![])] tiling_containers: Vec<TilingContainer>,
    #[builder(default = vec![])] non_tiling_windows: Vec<NonTilingWindow>,
  ) -> Self {
    let config = WorkspaceConfig {
      name,
      display_name,
      bind_to_monitor: None,
      keep_alive: false,
    };

    let workspace = Self::new(config, gaps_config, tiling_direction);

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
}
