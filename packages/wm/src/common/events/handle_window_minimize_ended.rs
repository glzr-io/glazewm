use tracing::info;

use crate::{
  common::platform::NativeWindow, containers::traits::CommonGetters, try_warn, user_config::UserConfig, windows::{
    commands::update_window_state, traits::WindowGetters, WindowState,
  }, wm_state::WmState
};

pub fn handle_window_minimize_ended(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  // Update the window's state to not be minimized.
  if let Some(window) = found_window {
    let is_minimized = try_warn!(window.native().refresh_is_minimized());

    if !is_minimized && window.state() == WindowState::Minimized {
      // TODO: Log window details.
      info!("Window minimize ended");
      
      // Handle minimizing fullscreen window if another non-floating window container is ending being minimized
      if let Some(fullscreen_window) = window.workspace().unwrap().get_fullscreen_window() {
        if !matches!(window.state(), WindowState::Floating(_)) {
          update_window_state(fullscreen_window, WindowState::Minimized, state, config)?;
        }
      }

      let target_state = window
        .prev_state()
        .unwrap_or(WindowState::default_from_config(config));

      update_window_state(window.clone(), target_state, state, config)?;
    }
  }

  Ok(())
}
