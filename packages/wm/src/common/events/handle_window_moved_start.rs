use anyhow::Context;
use tracing::info;

use crate::{
  containers::{
    commands::{attach_container, detach_container},
    traits::CommonGetters,
    Container,
  },
  user_config::{FloatingStateConfig, UserConfig},
  windows::{
    commands::update_window_state,
    traits::WindowGetters,
    window_operation::{
      Operation, WindowOperation,
    },
    TilingWindow, WindowState,
  },
  wm_state::WmState,
};

/// Handles window move events
pub fn window_moved_start(
  moved_window: TilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Tiling window drag start");

  moved_window.set_window_operation(WindowOperation {
    operation: Operation::Waiting
  });
  Ok(())
}
