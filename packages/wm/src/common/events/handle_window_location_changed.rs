use anyhow::Context;
use tracing::info;

use crate::{
  common::platform::NativeWindow,
  containers::{
    commands::move_container_within_tree, traits::CommonGetters,
  },
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
        // A window's location is changed when it gets minimized, so ignore
        // the event if the window is currently minimized.
        // TODO: Add fullscreen check.
        if !window.native().is_maximized()
          && !window.native().is_minimized()
        {
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
              ..config.value.window_behavior.state_defaults.fullscreen
            }),
            state,
            config,
          )?;
        } else if matches!(window.state(), WindowState::Floating(_)) {
          // Update state with the new location of the floating window.
          let new_position = window.native().frame_position()?;
          window.set_floating_placement(new_position);

          let workspace = window.workspace().context("No workspace.")?;

          // Get workspace that encompasses most of the window after moving.
          let updated_workspace = state
            .nearest_monitor(&window.native())
            .and_then(|monitor| monitor.displayed_workspace())
            .context("Failed to get workspace of nearest monitor.")?;

          // Update the window's workspace if it goes out of bounds of its
          // current workspace.
          if workspace.id() != updated_workspace.id() {
            info!("Window moved to new workspace.");

            move_container_within_tree(
              window.into(),
              updated_workspace.clone().into(),
              updated_workspace.child_count(),
              state,
            )?;
          }
        }
      }
    }
  }

  Ok(())
}
