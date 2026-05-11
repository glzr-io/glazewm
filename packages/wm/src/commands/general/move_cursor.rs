use crate::{
  traits::{CommonGetters, PositionGetters},
  wm_state::WmState,
};

/// Moves the cursor to the center of the currently focused window.
///
/// Does nothing if no window is currently focused (e.g. an empty workspace
/// is focused instead).
pub fn move_cursor_to_active_window(
  state: &WmState,
) -> anyhow::Result<()> {
  let Some(focused) = state.focused_container() else {
    return Ok(());
  };

  if focused.as_window_container().is_err() {
    return Ok(());
  }

  let center = focused.to_rect()?.center_point();

  if let Err(err) = state.dispatcher.set_cursor_position(&center) {
    tracing::warn!("Failed to move cursor: {}.", err);
  }

  Ok(())
}
