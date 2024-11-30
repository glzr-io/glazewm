use crate::{wm_event::WmEvent, wm_state::WmState};

/// Pauses or unpauses the WM.
pub fn toggle_pause(state: &mut WmState) -> anyhow::Result<()> {
  let new_paused_state = !state.is_paused;
  state.is_paused = new_paused_state;

  // Redraw full container tree on unpause.
  if !new_paused_state {
    let root_container = state.root_container.clone();
    state
      .pending_sync
      .containers_to_redraw
      .push(root_container.into());
  }

  state.emit_event(WmEvent::PauseChanged {
    is_paused: new_paused_state,
  });

  Ok(())
}
