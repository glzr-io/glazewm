use wm_common::WmEvent;

use crate::wm_state::WmState;

/// Pauses or unpauses the WM.
pub fn toggle_pause(state: &mut WmState) {
  let is_paused = !state.is_paused;

  // Redraw full container tree on unpause.
  if !is_paused {
    let root_container = state.root_container.clone();
    state
      .pending_sync
      .containers_to_redraw
      .push(root_container.into());
  }

  state.emit_event(WmEvent::PauseChanged { is_paused });
  state.is_paused = is_paused;
}
