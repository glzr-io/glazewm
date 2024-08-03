use anyhow::Context;
use tracing::info;

use crate::{
  common::{platform::NativeWindow, Rect},
  containers::{
    commands::{
      attach_container, detach_container, move_container_within_tree,
    },
    traits::{CommonGetters, PositionGetters},
    Container, WindowContainer,
  },
  user_config::{FloatingStateConfig, FullscreenStateConfig, UserConfig},
  windows::{
    commands::update_window_state, traits::WindowGetters,
    ActiveDragOperation, TilingWindow, WindowState,
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
    let frame_position: Rect = window.native().refresh_frame_position()?;
    let old_frame_position: Rect = window.to_rect()?;

    update_window_operation(
      state,
      config,
      &window,
      &frame_position,
      &old_frame_position,
    )?;

    let is_minimized = window.native().refresh_is_minimized()?;

    let old_is_maximized = window.native().is_maximized()?;
    let is_maximized = window.native().refresh_is_maximized()?;

    let nearest_monitor = state
      .nearest_monitor(&window.native())
      .context("Failed to get workspace of nearest monitor.")?;

    match window.state() {
      WindowState::Fullscreen(fullscreen_state) => {
        let monitor_rect = if config.has_outer_gaps() {
          nearest_monitor.native().working_rect()?.clone()
        } else {
          nearest_monitor.to_rect()?
        };

        let is_fullscreen =
          window.native().is_fullscreen(&monitor_rect)?;

        // A fullscreen window that gets minimized can hit this arm, so
        // ignore such events and let it be handled by the handler for
        // `PlatformEvent::WindowMinimized` instead.
        if !(is_fullscreen || is_maximized) && !is_minimized {
          info!("Window restored");

          let target_state = window
            .prev_state()
            .unwrap_or(WindowState::default_from_config(config));

          update_window_state(
            window.clone(),
            target_state,
            state,
            config,
          )?;
        } else if is_maximized != old_is_maximized {
          info!("Updating window's fullscreen state.");

          update_window_state(
            window.clone(),
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: is_maximized,
              ..fullscreen_state
            }),
            state,
            config,
          )?;
        }
      }
      _ => {
        // Update the window to be fullscreen if there's been a change in
        // maximized state or if the window is now fullscreen.
        if (is_maximized && old_is_maximized != is_maximized)
          || window
            .native()
            .is_fullscreen(nearest_monitor.native().working_rect()?)?
        {
          info!("Window fullscreened");

          update_window_state(
            window,
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: is_maximized,
              ..config.value.window_behavior.state_defaults.fullscreen
            }),
            state,
            config,
          )?;

        // A floating window that gets minimized can hit this arm, so
        // ignore such events and let it be handled by the handler for
        // `PlatformEvent::WindowMinimized` instead.
        } else if !is_minimized
          && matches!(window.state(), WindowState::Floating(_))
        {
          // Update state with the new location of the floating window.
          info!("Updating floating window position.");
          window.set_floating_placement(frame_position);

          let monitor = window.monitor().context("No monitor.")?;

          // Update the window's workspace if it goes out of bounds of its
          // current workspace.
          if monitor.id() != nearest_monitor.id() {
            let updated_workspace = nearest_monitor
              .displayed_workspace()
              .context("Failed to get workspace of nearest monitor.")?;

            info!(
              "Floating window moved to new workspace: '{}'.",
              updated_workspace.config().name
            );

            if let WindowContainer::NonTilingWindow(window) = &window {
              window.set_insertion_target(None);
            }

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

/// Updates the window operation based on changes in frame position.
///
/// This function determines whether a window is being moved or resized and
/// updates its operation state accordingly. If the window is being moved,
/// it's set to floating mode.
fn update_window_operation(
  state: &mut WmState,
  config: &UserConfig,
  window: &WindowContainer,
  frame_position: &Rect,
  old_frame_position: &Rect,
) -> anyhow::Result<()> {
  if let Some(tiling_window) = window.as_tiling_window() {
    if let Some(mut active_drag) = tiling_window.active_drag() {
      if active_drag.operation.is_none()
        && frame_position != old_frame_position
      {
        if frame_position.height() == old_frame_position.height()
          && frame_position.width() == old_frame_position.width()
        {
          active_drag.operation = Some(ActiveDragOperation::Moving);
          tiling_window.set_active_drag(Some(active_drag));
          set_into_floating(tiling_window.clone(), state, config)?;
        } else {
          active_drag.operation = Some(ActiveDragOperation::Resizing);
          tiling_window.set_active_drag(Some(active_drag));
        }
      }
    }
  }
  Ok(())
}

/// Converts a tiling window to a floating window and updates the window
/// hierarchy.
///
/// This function handles the process of transitioning a tiling window to a
/// floating state, including necessary adjustments to the window hierarchy
/// and updating the window's state.
fn set_into_floating(
  moved_window: TilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let moved_window_parent = moved_window
    .parent()
    .context("Tiling window has no parent")?;

  if let Some(Container::Split(split)) = moved_window.parent() {
    if split.child_count() == 2 {
      let split_parent = split.parent().unwrap();
      let split_index = split.index();
      let children = split.children();

      // Looping in reversed order to reattach them in the right order
      for child in children.into_iter().rev() {
        detach_container(child.clone())?;
        attach_container(&child, &split_parent, Some(split_index))?;
      }
    }
  }

  if let Some(mut active_drag) = moved_window.active_drag() {
    active_drag.is_from_tiling = true;
    moved_window.set_active_drag(Some(active_drag));
  }

  update_window_state(
    moved_window.as_window_container().unwrap(),
    WindowState::Floating(FloatingStateConfig {
      centered: true,
      shown_on_top: true,
    }),
    state,
    config,
  )?;
  state
    .pending_sync
    .containers_to_redraw
    .push(moved_window_parent);
  Ok(())
}
