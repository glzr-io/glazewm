use crate::{
  user_config::UserConfig, wm_event::WmEvent, wm_state::WmState,
};

/// Pauses or unpauses the WM.
pub fn toggle_pause(state: &mut WmState) -> anyhow::Result<()> {
  let new_paused_state = !state.paused;
  state.paused = new_paused_state;

  state.emit_event(WmEvent::PauseChanged {
    paused: new_paused_state,
  });

  Ok(())
}
