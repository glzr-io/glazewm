use std::{collections::VecDeque, io::Split};

use anyhow::Context;
use tracing::{debug, info};
use windows::Win32::{Foundation, UI::WindowsAndMessaging::GetCursorPos};

use crate::{
  common::{
    commands::platform_sync,
    LengthValue,
    platform::{MouseMoveEvent, NativeWindow, Platform}, Point, Rect, TilingDirection,
  },
  containers::{
    commands::{
      attach_container, detach_container, move_container_within_tree,
    },
    Container,
    SplitContainer, traits::{CommonGetters, PositionGetters, TilingDirectionGetters}, WindowContainer,
  },
  user_config::UserConfig,
  windows::{
    commands::resize_window, NonTilingWindow, TilingWindow,
    traits::WindowGetters,
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
  info!("Tiling window move start");
  Ok(())
}
