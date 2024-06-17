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
    info!("Tiling window moved/resized");

    let parent = window.parent().context("No parent.")?;

    // Snap window to its original position if it's the only window in the
    // workspace.
    if parent.is_workspace() && window.tiling_siblings().count() == 0 {
      state.pending_sync.containers_to_redraw.push(window.into());
      return Ok(());
    }

    let frame_position = window.native().refresh_frame_position()?;
    let width_delta = frame_position.width() - window.width()?;
    let height_delta = frame_position.height() - window.height()?;

    resize_window(
      window.clone().into(),
      Some(LengthValue::new_px(width_delta as f32)),
      Some(LengthValue::new_px(height_delta as f32)),
      state,
    )?;
  }

  Ok(())
}
