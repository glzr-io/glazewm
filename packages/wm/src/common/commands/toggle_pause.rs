use crate::{
  user_config::{UserConfig, WindowRuleEvent},
  windows::{commands::run_window_rules, traits::WindowGetters},
  wm_event::WmEvent,
  wm_state::WmState,
};

/// Pauses or unpauses the WM.
pub fn toggle_pause(state: &mut WmState) -> anyhow::Result<()> {
  let new_paused_state = !state.paused;
  state.paused = new_paused_state;

  if !new_paused_state {
    // Redraw full container tree.
    let root_container = state.root_container.clone();
    state
      .pending_sync
      .containers_to_redraw
      .push(root_container.into());
  }

  state.emit_event(WmEvent::PauseChanged {
    paused: new_paused_state,
  });

  Ok(())
}
