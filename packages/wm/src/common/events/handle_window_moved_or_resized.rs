use anyhow::Context;
use tracing::info;

use crate::{
  common::{platform::NativeWindow, LengthValue},
  containers::{
    traits::{CommonGetters, PositionGetters},
    WindowContainer,
  },
  windows::{commands::resize_window, traits::WindowGetters},
  wm_state::WmState,
};

/// Handles the event for when a window is finished being moved or resized
/// by the user (e.g. via the window's drag handles).
///
/// This resizes the window if it's a tiling window.
pub fn handle_window_moved_or_resized(
  native_window: NativeWindow,
  state: &mut WmState,
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

    let window_moved = match (width_delta, height_delta) {
      (0, 0) => true,
      _ => false,
    };

    match window_moved {
      true => info!("Tiling window moved"),
      false => info!("Tiling window resized"),
    }

    resize_window(
      window.clone().into(),
      Some(LengthValue::from_px(width_delta)),
      Some(LengthValue::from_px(height_delta)),
      state,
    )?;
  }

  Ok(())
}
