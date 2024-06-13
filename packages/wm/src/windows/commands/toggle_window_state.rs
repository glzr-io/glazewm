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
/// Unlike `update_window_state`, this will always add the window for
/// redraw.
pub fn toggle_window_state(
  window: WindowContainer,
  target_window_state: WindowState,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let toggled_window_state = match window.state() == target_window_state {
    true => window.prev_state().unwrap_or(WindowState::Tiling),
    false => target_window_state,
  };

  match toggled_window_state {
    WindowState::Minimized => window.native().minimize(),
    _ => {
      update_window_state(
        window.clone(),
        toggled_window_state,
        state,
        config,
      )?;

      // The `update_window_state` call will already add the window for redraw
      // on tiling <-> non-tiling state changes. However, on toggle, we always
      // want to add the window for redraw.
      state.pending_sync.containers_to_redraw.push(window.into());
      Ok(())
    }
  }
}
