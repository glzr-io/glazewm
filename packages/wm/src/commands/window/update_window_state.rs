use anyhow::Context;
use tracing::{info, warn};
use wm_common::WindowState;

use crate::{
  commands::{
    container::{
      attach_container, move_container_within_tree, replace_container,
    },
    window::manage_window::rebuild_spiral_layout,
  },
  models::{InsertionTarget, TilingWindow, WindowContainer},
  traits::{CommonGetters, TilingSizeGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Updates the state of a window.
///
/// Adds the window for redraw if there is a state change.
///
/// Returns the window after the state change.
pub fn update_window_state(
  window: WindowContainer,
  target_state: WindowState,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WindowContainer> {
  if window.state() == target_state {
    return Ok(window);
  }

  info!("Updating window state: {:?}.", target_state);

  match target_state {
    WindowState::Tiling => set_tiling(&window, state, config),
    _ => set_non_tiling(window, target_state, state),
  }
}

/// Updates the state of a window to be `WindowState::Tiling`.
fn set_tiling(
  window: &WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WindowContainer> {
  let window = window
    .as_non_tiling_window()
    .context("Invalid window state.")?
    .clone();

  let workspace =
    window.workspace().context("Window has no workspace.")?;

  let tiling_window = window.to_tiling(config.value.gaps.clone());

  // Replace the original window with the created tiling window.
  let parent = match window.parent() {
    Some(parent) => parent,
    None => {
      // Window is detached, just attach to workspace directly
      attach_container(
        &tiling_window.clone().into(),
        &workspace.clone().into(),
        Some(workspace.child_count()),
      )?;

      // Force a spiral rebuild to integrate the restored window correctly
      let tiling_windows: Vec<TilingWindow> = workspace
        .descendants()
        .filter_map(|c| c.try_into().ok())
        .collect();

      if !tiling_windows.is_empty() {
        rebuild_spiral_layout(&workspace, &tiling_windows)?;
      }

      state
        .pending_sync
        .queue_containers_to_redraw(workspace.tiling_children())
        .queue_workspace_to_reorder(workspace);

      return Ok(tiling_window.into());
    }
  };

  replace_container(
    &tiling_window.clone().into(),
    &parent,
    window.index(),
  )?;

  // Append the new tiling window to the workspace
  move_container_within_tree(
    &tiling_window.clone().into(),
    &workspace.clone().into(),
    workspace.child_count(),
    state,
  )?;

  // Force a spiral rebuild to integrate the restored window correctly
  let tiling_windows: Vec<TilingWindow> = workspace
    .descendants()
    .filter_map(|c| c.try_into().ok())
    .collect();

  if !tiling_windows.is_empty() {
    rebuild_spiral_layout(&workspace, &tiling_windows)?;
  }

  state
    .pending_sync
    .queue_containers_to_redraw(workspace.tiling_children())
    .queue_workspace_to_reorder(workspace);

  Ok(tiling_window.into())
}

/// Updates the state of a window to be either `WindowState::Floating`,
/// `WindowState::Fullscreen`, or `WindowState::Minimized`.
fn set_non_tiling(
  window: WindowContainer,
  target_state: WindowState,
  state: &mut WmState,
) -> anyhow::Result<WindowContainer> {
  // A window can only be updated to a minimized state if it is
  // natively minimized.
  if target_state == WindowState::Minimized
    && !window.native().is_minimized()?
  {
    info!("No window state update. Minimizing window.");

    if let Err(err) = window.native().minimize() {
      warn!("Failed to minimize window: {}", err);
    }

    return Ok(window);
  }

  let workspace = window.workspace().context("No workspace.")?;

  match window {
    WindowContainer::NonTilingWindow(window) => {
      let current_state = window.state();

      // Update the window's previous state if the discriminant changes.
      if !current_state.is_same_state(&target_state) {
        window.set_prev_state(current_state);
        state.pending_sync.queue_workspace_to_reorder(workspace);
      }

      window.set_state(target_state);
      state.pending_sync.queue_container_to_redraw(window.clone());

      Ok(window.into())
    }
    WindowContainer::TilingWindow(window) => {
      let parent = match window.parent() {
        Some(parent) => parent,
        None => {
          // Window is detached, just convert to non-tiling and attach to workspace
          let non_tiling_window = window.to_non_tiling(
            target_state.clone(),
            Some(InsertionTarget {
              target_parent: workspace.clone().into(),
              target_index: workspace.child_count(),
              prev_tiling_size: window.tiling_size(),
              prev_sibling_count: window.tiling_siblings().count(),
            }),
          );

          attach_container(
            &non_tiling_window.clone().into(),
            &workspace.clone().into(),
            Some(workspace.child_count()),
          )?;

          state
            .pending_sync
            .queue_container_to_redraw(non_tiling_window.clone())
            .queue_workspace_to_reorder(workspace);

          return Ok(non_tiling_window.into());
        }
      };

      let non_tiling_window = window.to_non_tiling(
        target_state.clone(),
        Some(InsertionTarget {
          target_parent: parent.clone(),
          target_index: window.index(),
          prev_tiling_size: window.tiling_size(),
          prev_sibling_count: window.tiling_siblings().count(),
        }),
      );

      // Non-tiling windows should always be direct children of the
      // workspace.
      if parent != workspace.clone().into() {
        move_container_within_tree(
          &window.clone().into(),
          &workspace.clone().into(),
          workspace.child_count(),
          state,
        )?;
      }

      replace_container(
        &non_tiling_window.clone().into(),
        &workspace.clone().into(),
        window.index(),
      )?;

      // Rebuild spiral layout for remaining windows to heal the gap
      let remaining_windows: Vec<TilingWindow> = workspace
        .descendants()
        .filter_map(|c| c.try_into().ok())
        .collect();

      if !remaining_windows.is_empty() {
        rebuild_spiral_layout(&workspace, &remaining_windows)?;
      }

      state
        .pending_sync
        .queue_container_to_redraw(non_tiling_window.clone())
        .queue_containers_to_redraw(workspace.tiling_children())
        .queue_workspace_to_reorder(workspace);

      Ok(non_tiling_window.into())
    }
  }
}
