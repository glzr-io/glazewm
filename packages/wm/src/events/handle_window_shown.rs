use tracing::info;
use wm_common::DisplayState;
use wm_platform::NativeWindow;

use crate::{
  commands::window::manage_window, traits::WindowGetters,
  user_config::UserConfig, wm_state::WmState,
};

pub fn handle_window_shown(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  match found_window {
    Some(window) => {
      info!("Window shown: {window}");

      // Update display state if window is already managed.
      if window.display_state() == DisplayState::Showing {
        window.set_display_state(DisplayState::Shown);
      } else {
        state.pending_sync.queue_container_to_redraw(window);
      }
    }
    None => {
      // If the window is not managed, manage it.
      if native_window.is_manageable().unwrap_or(false) {
        manage_window(native_window, None, state, config)?;
      }
    }
  }

  Ok(())
}
