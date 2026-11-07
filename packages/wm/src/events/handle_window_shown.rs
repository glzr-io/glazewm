use tracing::info;
use wm_common::{DisplayState, HideMethod};
use wm_platform::NativeWindow;

use crate::{
  commands::window::{manage_window, restore_parked_window},
  traits::WindowGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_window_shown(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  if let Some(window) = found_window {
    info!("Window shown: {window}");

    // Update display state if window is already managed.
    if config.value.general.hide_method != HideMethod::PlaceInCorner
      && window.display_state() == DisplayState::Showing
    {
      window.set_display_state(DisplayState::Shown);
    } else {
      state.pending_sync.queue_container_to_redraw(window);
    }
  } else if let Some(parked_index) = state
    .parked_windows
    .iter()
    .position(|parked| *parked.window.native() == native_window)
  {
    // Restore window that was previously moved off the current virtual
    // desktop. Shown events can be received for windows that are still on
    // another virtual desktop, so ensure the window is actually visible.
    if native_window.is_visible().unwrap_or(false) {
      let parked = state.parked_windows.remove(parked_index);

      // Manage as a new window if its previous workspace no longer exists.
      if restore_parked_window(parked, state)?.is_some() {
        manage_window(native_window, None, state, config)?;
      }
    }
  } else if !state.ignored_windows.contains(&native_window) {
    // If the window is not managed and not explicitly ignored, manage it.
    manage_window(native_window, None, state, config)?;
  }

  Ok(())
}
