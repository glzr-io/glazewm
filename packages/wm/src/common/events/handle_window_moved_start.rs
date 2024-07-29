use std::{collections::VecDeque, io::Split};

use anyhow::Context;
use tracing::{debug, info};
use windows::Win32::{Foundation, UI::WindowsAndMessaging::GetCursorPos};

use crate::{
  common::{
    commands::platform_sync,
    platform::{MouseMoveEvent, NativeWindow, Platform},
    LengthValue, Point, Rect, TilingDirection,
  },
  containers::{
    commands::{
      attach_container, detach_container, move_container_within_tree,
    },
    traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
    Container, SplitContainer, WindowContainer,
  },
  user_config::{FloatingStateConfig, UserConfig},
  windows::{
    commands::{resize_window, update_window_state},
    traits::WindowGetters,
    NonTilingWindow, TilingWindow, WindowState,
  },
  wm_event::WmEvent,
  wm_state::WmState,
};

/// Handles window move events
pub fn window_moved_start(
  moved_window: TilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Tiling window drag start");
  let moved_window_parent = moved_window
    .parent()
    .context("Tiling window has no parent")?;

  update_window_state(
    moved_window.as_window_container().unwrap(),
    WindowState::Floating(FloatingStateConfig {
      centered: true,
      shown_on_top: true,
      is_tiling_drag: true,
    }),
    state,
    config,
  )?;
  state
    .pending_sync
    .containers_to_redraw
    .push(moved_window_parent);
  Ok(())
}
