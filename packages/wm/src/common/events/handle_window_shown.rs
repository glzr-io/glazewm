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

  match found_window {
    Some(window) => {
      // TODO: Log window details.
      info!("Showing window");

      // Update display state if window is already managed.
      if window.display_state() == DisplayState::Showing {
        window.set_display_state(DisplayState::Shown);
      }
    }
    None => {
      // If the window is not managed, manage it.
      if native_window.is_manageable().unwrap_or(false) {
        manage_window(native_window, None, state, config)?;
      }
    }
  };

  Ok(())
}
