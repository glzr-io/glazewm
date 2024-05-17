use crate::{
  containers::WindowContainer,
  user_config::UserConfig,
  windows::{traits::WindowGetters, WindowState},
  wm_state::WmState,
};

use super::update_window_state;

/// Toggles the state of a window between its previous state if the window
/// has one, or tiling if it does not.
///
/// Always adds the window for redraw.
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
      update_window_state(window.clone(), prev_state, state, config)
    }
    None if window.state() != WindowState::Tiling => update_window_state(
      window.clone(),
      WindowState::Tiling,
      state,
      config,
    ),
    None => Ok(()),
  }?;

  // The `update_window_state` call will already add the window for redraw
  // on tiling <-> non-tiling state changes. However, on toggle, we always
  // want to add the window for redraw.
  state.containers_to_redraw.push(window.into());

  Ok(())
}
