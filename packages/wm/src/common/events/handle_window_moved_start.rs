use anyhow::Context;
use tracing::info;

use crate::{
  containers::{
    commands::{attach_container, detach_container},
    traits::CommonGetters,
    Container,
  },
  user_config::{FloatingStateConfig, UserConfig},
  windows::{commands::update_window_state, TilingWindow, WindowState},
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

  if let Some(Container::Split(split)) = moved_window.parent() {
    if split.child_count() == 1 {
      let split_parent = split.parent().unwrap();
      let split_index = split.index();
      detach_container(Container::TilingWindow(moved_window.clone()))?;
      attach_container(
        &Container::TilingWindow(moved_window.clone()),
        &split_parent,
        Some(split_index),
      )?;
    }
  }

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
