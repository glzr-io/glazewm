use anyhow::Context;
use tracing::info;

use crate::{
  common::{
    platform::{NativeWindow, Platform},
    LengthValue, TilingDirection,
  },
  containers::{
    commands::move_container_within_tree,
    traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
    DirectionContainer, TilingContainer, WindowContainer,
  },
  try_warn,
  user_config::UserConfig,
  windows::{
    commands::{resize_window, update_window_state},
    traits::WindowGetters,
    ActiveDragOperation, NonTilingWindow, WindowState,
  },
  wm_state::WmState,
};

/// Handles the event for when a window is finished being moved or resized
/// by the user (e.g. via the window's drag handles).
///
/// This resizes the window if it's a tiling window and attach a dragged
/// floating window.
pub fn handle_window_moved_or_resized_end(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  if let Some(window) = found_window {
    // TODO: Log window details.

    let new_rect = try_warn!(window.native().refresh_frame_position());
    let old_rect = window.to_rect()?;

    let width_delta = new_rect.width() - old_rect.width();
    let height_delta = new_rect.height() - old_rect.height();

    match &window {
      WindowContainer::NonTilingWindow(window) => {
        if let Some(active_drag) = window.active_drag() {
          if active_drag.is_from_tiling
            && active_drag.operation == Some(ActiveDragOperation::Moving)
          {
            // Window is a temporary floating window that should be
            // reverted back to tiling.
            drop_as_tiling_window(window.clone(), state, config)?;
          }
        }
      }
      WindowContainer::TilingWindow(window) => {
        info!("Tiling window resized");

        let parent = window.parent().context("No parent.")?;

        // Snap window to its original position if it's the only window in
        // the workspace.
        if parent.is_workspace() && window.tiling_siblings().count() == 0 {
          state
            .pending_sync
            .containers_to_redraw
            .push(window.clone().into());

          return Ok(());
        }

        resize_window(
          window.clone().into(),
          Some(LengthValue::from_px(width_delta)),
          Some(LengthValue::from_px(height_delta)),
          state,
        )?;

        state.pending_sync.containers_to_redraw.push(parent);
      }
    }

    window.set_active_drag(None);
  }

  Ok(())
}

/// Handles transition from temporary floating window to tiling window on
/// drag end.
fn drop_as_tiling_window(
  moved_window: NonTilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Tiling window drag end event.");

  let mouse_pos = Platform::mouse_position()?;

  // Get the workspace, split containers, and other windows under the
  // dragged window.
  let containers_at_pos = state
    .containers_at_point(&mouse_pos)
    .into_iter()
    .filter(|container| container.id() != moved_window.id());

  let workspace = moved_window.workspace().context("No workspace.")?;

  // Get the deepest direction container under the dragged window.
  let target_parent: DirectionContainer = containers_at_pos
    .filter_map(|container| container.as_direction_container().ok())
    .fold(workspace.into(), |acc, container| {
      if container.ancestors().count() > acc.ancestors().count() {
        container
      } else {
        acc
      }
    });

  // If the target parent has no children (i.e. an empty workspace), then
  // add the window directly.
  if target_parent.tiling_children().count() == 0 {
    update_window_state(
      moved_window.clone().into(),
      WindowState::Tiling,
      state,
      config,
    )?;

    return Ok(());
  }

  let tiling_direction = target_parent.tiling_direction();

  let nearest_container = target_parent
    .children()
    .into_iter()
    .filter_map(|container| container.as_tiling_container().ok())
    .try_fold(None, |acc: Option<TilingContainer>, container| {
      let distance = |container: &TilingContainer| -> anyhow::Result<i32> {
        let rect = container.to_rect()?;

        Ok(match tiling_direction {
          TilingDirection::Horizontal => (rect.x() - mouse_pos.x)
            .abs()
            .min((rect.x() + rect.width() - mouse_pos.x).abs()),
          TilingDirection::Vertical => (rect.y() - mouse_pos.y)
            .abs()
            .min((rect.y() + rect.height() - mouse_pos.y).abs()),
        })
      };

      match acc {
        Some(acc) => {
          let is_nearer = distance(&acc)? < distance(&container)?;
          anyhow::Ok(Some(if is_nearer { acc } else { container }))
        }
        None => Ok(Some(container)),
      }
    })?
    .context("No nearest container.")?;

  let target_index = match tiling_direction {
    TilingDirection::Horizontal => {
      match mouse_pos.x < nearest_container.to_rect()?.center_point().x {
        true => nearest_container.index(),
        false => nearest_container.index() + 1,
      }
    }
    TilingDirection::Vertical => {
      match mouse_pos.y < nearest_container.to_rect()?.center_point().y {
        true => nearest_container.index(),
        false => nearest_container.index() + 1,
      }
    }
  };

  let moved_window = update_window_state(
    moved_window.clone().into(),
    WindowState::Tiling,
    state,
    config,
  )?;

  move_container_within_tree(
    moved_window.into(),
    target_parent.clone().into(),
    target_index,
    state,
  )?;

  state
    .pending_sync
    .containers_to_redraw
    .push(target_parent.into());

  Ok(())
}
