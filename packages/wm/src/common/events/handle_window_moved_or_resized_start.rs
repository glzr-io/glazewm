use anyhow::Context;
use tracing::info;

use crate::{
  common::{
    events::handle_window_moved_start::window_moved_start,
    platform::NativeWindow,
  },
  containers::{
    traits::{CommonGetters, PositionGetters},
    WindowContainer,
  },
  user_config::UserConfig,
  windows::traits::WindowGetters,
  wm_state::WmState,
};

/// Handles the event for when a window is finished being moved or resized
/// by the user (e.g. via the window's drag handles).
///
/// This resizes the window if it's a tiling window.
pub fn handle_window_moved_or_resized_start(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  if let Some(WindowContainer::TilingWindow(window)) = found_window {
    // TODO: Log window details.

    let parent = window.parent().context("No parent.")?;

    // Snap window to its original position if it's the only window in the
    // workspace.
    if parent.is_workspace() && window.tiling_siblings().count() == 0 {
      state.pending_sync.containers_to_redraw.push(window.into());
      return Ok(());
    }

    let new_rect = window.native().refresh_frame_position()?;
    let old_rect = window.to_rect()?;

    let width_delta = new_rect.width() - old_rect.width();
    let height_delta = new_rect.height() - old_rect.height();

    let has_window_moved = match (width_delta, height_delta) {
      (0, 0) => true,
      _ => false,
    };

    return match has_window_moved {
      true => window_moved_start(window, state, config),
      false => window_resized_start(),
    };
  }

  Ok(())
}

/// Handles window resize events
fn window_resized_start() -> anyhow::Result<()> {
  info!("Tiling window resize start");
  Ok(())
}
