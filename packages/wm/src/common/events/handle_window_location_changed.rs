use anyhow::Context;
use tracing::info;

use crate::{
  common::platform::NativeWindow,
  containers::{
    commands::move_container_within_tree,
    traits::{CommonGetters, PositionGetters},
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
    window.native().refresh_border_position()?;
    window.native().refresh_frame_position()?;
    window.native().refresh_is_maximized()?;

    let nearest_monitor = state
      .nearest_monitor(&window.native())
      .context("Failed to get workspace of nearest monitor.")?;

    match window.state() {
      WindowState::Fullscreen(_) => {
        window.native().refresh_is_minimized()?;

        // A fullscreen window that gets minimized can hit this arm, so
        // ignore such events and let it be handled by the handler for
        // `PlatformEvent::WindowMinimized` instead.
        if !window.native().is_fullscreen(&nearest_monitor.to_rect()?)?
          && !window.native().is_maximized()?
          && !window.native().is_minimized()?
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
        if window.native().is_maximized()?
          || window.native().is_fullscreen(&nearest_monitor.to_rect()?)?
        {
          info!("Window fullscreened");
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
          let updated_workspace = nearest_monitor
            .displayed_workspace()
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
