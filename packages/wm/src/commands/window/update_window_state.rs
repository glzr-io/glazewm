use anyhow::Context;
use tracing::info;
use wm_common::WindowState;

use crate::{
  commands::container::{
    move_container_within_tree, replace_container, resize_tiling_container,
  },
  models::{Container, InsertionTarget, WindowContainer},
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

  // Check whether insertion target is still valid.
  let insertion_target =
    window.insertion_target().filter(|insertion_target| {
      insertion_target
        .target_parent
        .workspace()
        .is_some_and(|workspace| workspace.is_displayed())
    });

  // Get the position in the tree to insert the new tiling window. This
  // will be the window's previous tiling position if it has one, or
  // instead beside the last focused tiling window in the workspace.
  let (target_parent, target_index) = insertion_target
    .as_ref()
    .map(|insertion_target| {
      (
        insertion_target.target_parent.clone(),
        insertion_target.target_index,
      )
    })
    // Fallback to the last focused tiling window within the workspace.
    .or_else(|| {
      let focused_window = workspace
        .descendant_focus_order()
        .find(Container::is_tiling_window)?;

      Some((focused_window.parent()?, focused_window.index() + 1))
    })
    // Default to inserting at the end of the workspace.
    .unwrap_or((workspace.clone().into(), workspace.child_count()));

  let tiling_window = window.to_tiling(config.value.gaps.clone());

  // Replace the original window with the created tiling window.
  replace_container(
    &tiling_window.clone().into(),
    &window.parent().context("No parent.")?,
    window.index(),
  )?;

  move_container_within_tree(
    tiling_window.clone().into(),
    &target_parent,
    target_index,
    state,
  )?;

  if let Some(insertion_target) = &insertion_target {
    let size_scale = (insertion_target.prev_sibling_count + 1) as f32
      / (tiling_window.tiling_siblings().count() + 1) as f32;

    // Scale the window's previous size based on the current number of
    // siblings. E.g. if the window was 0.5 with 1 sibling, and now has 2
    // siblings, scale to 0.5 * (2/3) to maintain proportional sizing.
    let target_size = insertion_target.prev_tiling_size * size_scale;
    resize_tiling_container(&tiling_window.clone().into(), target_size);
  }

  state
    .pending_sync
    .containers_to_redraw
    .extend(target_parent.tiling_children().map(Into::into));

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
    window.native().minimize()?;
    return Ok(window);
  }

  match window {
    WindowContainer::NonTilingWindow(window) => {
      let current_state = window.state();

      // Update the window's previous state if the discriminant changes.
      if std::mem::discriminant(&current_state)
        != std::mem::discriminant(&target_state)
      {
        window.set_prev_state(current_state);
      }

      window.set_state(target_state);

      state
        .pending_sync
        .containers_to_redraw
        .push(window.clone().into());

      Ok(window.into())
    }
    WindowContainer::TilingWindow(window) => {
      let parent = window.parent().context("No parent")?;
      let workspace = window.workspace().context("No workspace.")?;

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
          window.clone().into(),
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

      let changed_containers =
        std::iter::once(non_tiling_window.clone().into())
          .chain(workspace.tiling_children().map(Into::into));

      state
        .pending_sync
        .containers_to_redraw
        .extend(changed_containers);

      Ok(non_tiling_window.into())
    }
  }
}
