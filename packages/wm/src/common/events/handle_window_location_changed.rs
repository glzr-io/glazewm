use tracing::info;

use crate::{
  common::platform::NativeWindow,
  user_config::{FullscreenStateConfig, UserConfig},
  windows::{
    commands::{toggle_window_state, update_window_state},
    traits::WindowGetters,
    WindowState,
  },
  wm_state::WmState,
};

pub fn handle_window_location_changed(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  // Update the window's state to be fullscreen or toggled from fullscreen.
  if let Some(window) = found_window {
    match window.state() {
      WindowState::Fullscreen(_) => {
        // TODO: Add fullscreen check.
        if !window.native().is_maximized() {
          info!("Window restored");
          toggle_window_state(
            window.clone(),
            window.state(),
            state,
            config,
          )?;
        }
      }
      _ => {
        if window.native().is_maximized()
          || window.native().is_fullscreen()
        {
          info!("Window maximized");
          update_window_state(
            window,
            WindowState::Fullscreen(FullscreenStateConfig {
              ..config.value.window_state_defaults.fullscreen
            }),
            state,
            config,
          )?;
        }
      }
    }
  }

  Ok(())
}
