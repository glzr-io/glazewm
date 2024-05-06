use tracing::info;

use crate::{
  common::platform::NativeWindow,
  user_config::UserConfig,
  windows::{
    commands::update_window_state, traits::WindowGetters, WindowState,
  },
  wm_state::WmState,
};

pub fn handle_window_minimized(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  // Update the window's state to be minimized.
  if let Some(window) = found_window {
    if window.state() != WindowState::Minimized {
      // TODO: Log window details.
      info!("Window minimized");
      update_window_state(window, WindowState::Minimized, state, config)?;
    }
  }

  Ok(())
}
