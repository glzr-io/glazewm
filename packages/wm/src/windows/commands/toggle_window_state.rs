use crate::{
  containers::WindowContainer,
  user_config::UserConfig,
  windows::{traits::WindowGetters, WindowState},
  wm_state::WmState,
};

use super::update_window_state;

pub fn toggle_window_state(
  window: WindowContainer,
  window_state: WindowState,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // If the window is not currently in the given state, update it to that
  // state.
  if window.state() != window_state {
    return update_window_state(window, window_state, state, config);
  }

  // Otherwise, revert to the window's previous state. If it doesn't have
  // a previous state, the window is updated to be tiling.
  match window.prev_state() {
    Some(prev_state) => {
      update_window_state(window, prev_state, state, config)
    }
    None if window.state() != WindowState::Tiling => {
      update_window_state(window, WindowState::Tiling, state, config)
    }
    None => Ok(()),
  }
}
