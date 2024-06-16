use anyhow::Context;
use tracing::info;

use crate::{
  common::{platform::NativeWindow, LengthValue, ResizeDimension},
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
    let delta_width = frame_position.width() - window.width()?;
    let delta_height = frame_position.height() - window.height()?;

    resize_window(
      window.clone().into(),
      ResizeDimension::Width,
      LengthValue::new_px(delta_width as f32),
      state,
    )?;

    resize_window(
      window.into(),
      ResizeDimension::Height,
      LengthValue::new_px(delta_height as f32),
      state,
    )?;
  }

  Ok(())
}
