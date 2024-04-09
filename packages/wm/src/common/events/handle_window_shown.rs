use tracing::info;

use crate::{
  common::{
    commands::sync_native_focus, platform::NativeWindow, DisplayState,
  },
  containers::commands::redraw,
  user_config::UserConfig,
  windows::{commands::manage_window, traits::WindowGetters},
  wm_state::WmState,
};

pub fn handle_window_shown(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // TODO: Refresh monitor state.
  if native_window.is_app_bar() {
    state.app_bar_windows.push(native_window);
    return Ok(());
  }

  let found_window = state.window_from_native(&native_window);

  // Manage the window if it's manageable.
  if found_window.is_none() && native_window.is_manageable() {
    manage_window(native_window, None, state, config)?;
    redraw(state, config)?;
    sync_native_focus(state)?;
    return Ok(());
  }

  if let Some(window) = found_window {
    // TODO: Log window details.
    info!("Showing window");

    // Update display state if window is already managed.
    if window.display_state() == DisplayState::Showing {
      window.set_display_state(DisplayState::Shown);
    }
  }
  Ok(())
}
