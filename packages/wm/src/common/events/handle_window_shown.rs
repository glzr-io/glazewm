use tracing::info;

use crate::{
  common::{platform::NativeWindow, DisplayState},
  user_config::UserConfig,
  windows::{commands::manage_window, traits::WindowGetters},
  wm_state::WmState,
};

pub fn handle_window_shown(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  // Manage the window if it's manageable.
  if found_window.is_none() && native_window.is_manageable() {
    manage_window(native_window, None, state, config)?;
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
