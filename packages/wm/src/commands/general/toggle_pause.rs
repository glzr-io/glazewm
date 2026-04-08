use wm_common::WmEvent;

use crate::wm_state::WmState;
#[cfg(target_os = "windows")]
use crate::{
  commands::general::sync_focused_window_border, user_config::UserConfig,
};

/// Pauses or unpauses the WM.
pub fn toggle_pause(
  state: &mut WmState,
  #[cfg(target_os = "windows")] config: &UserConfig,
) {
  let is_paused = !state.is_paused;
  state.is_paused = is_paused;

  // Redraw full container tree on unpause.
  if !is_paused {
    state
      .pending_sync
      .queue_container_to_redraw(state.root_container.clone());
  }

  #[cfg(target_os = "windows")]
  if let Err(err) = sync_focused_window_border(state, config) {
    tracing::warn!("Failed to sync Win10 focus highlight: {}", err);
  }

  state.emit_event(WmEvent::PauseChanged { is_paused });
}
