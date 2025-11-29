use anyhow::Context;
use tracing::info;
use wm_common::{
  try_warn, ActiveDragOperation, LengthValue, Point, Rect, WindowState,
};
use wm_platform::{NativeWindow, Platform};

use crate::{
  commands::{
    window::{manage_window::rebuild_spiral_layout, resize_window},
    window::update_window_state,
  },
  models::{NonTilingWindow, TilingWindow, WindowContainer},
  traits::{
    CommonGetters, PositionGetters, WindowGetters,
  },
  user_config::UserConfig,
  wm_state::WmState,
};

/// Handles the event for when a window is finished being moved or resized
/// by the user (e.g. via the window's drag handles).
///
/// This resizes the window if it's a tiling window and attach a dragged
/// floating window.
pub fn handle_window_moved_or_resized_end(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // Don't update state on resize events if the WM is paused.
  if state.is_paused {
    return Ok(());
  }

  let found_window = state.window_from_native(native_window);

  if let Some(window) = found_window {
    info!("Window move/resize ended: {window}");

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
            drop_as_tiling_window(window, state, config)?;
          }
        }
      }
      WindowContainer::TilingWindow(window) => {
        let parent = match window.parent() {
          Some(parent) => parent,
          None => {
            // Window is detached, just redraw it and return
            state.pending_sync.queue_container_to_redraw(window.clone());
            return Ok(());
          }
        };

        // Snap window to its original position if it's the only window in
        // the workspace.
        if parent.is_workspace() && window.tiling_siblings().count() == 0 {
          state.pending_sync.queue_container_to_redraw(window.clone());
          return Ok(());
        }

        resize_window(
          &window.clone().into(),
          Some(LengthValue::from_px(width_delta)),
          Some(LengthValue::from_px(height_delta)),
          state,
        )?;
      }
    }

    window.set_active_drag(None);
  }

  Ok(())
}

/// Handles transition from temporary floating window to tiling window on
/// drag end.
fn drop_as_tiling_window(
  moved_window: &NonTilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!(
    "Tiling window drag ended: {}",
    moved_window.as_window_container()?
  );

  let mouse_pos = Platform::mouse_position()?;
  let workspace = moved_window.workspace().context("No workspace.")?;

  // Find the tiling window under the cursor to use as a reference for reordering
  let target_window: Option<TilingWindow> = state
    .containers_at_point(&workspace.clone().into(), &mouse_pos)
    .into_iter()
    .find_map(|c| c.as_tiling_window().cloned());

  // Transition the moved window back to tiling state.
  // This will initially append it to the workspace and rebuild the spiral.
  let moved_window = update_window_state(
    moved_window.clone().into(),
    WindowState::Tiling,
    state,
    config,
  )?;

  // If we dropped onto another window, we need to reorder the spiral list
  if let Some(target_window) = target_window {
    if target_window.id() != moved_window.id() {
      let drop_position =
        drop_position(&mouse_pos, &target_window.to_rect()?);

      let insert_after = matches!(
        drop_position,
        DropPosition::Bottom | DropPosition::Right
      );

      // Get all tiling windows in their current spiral order
      let mut windows: Vec<TilingWindow> = workspace
        .descendants()
        .filter_map(|c| c.try_into().ok())
        .collect();

      // Remove the moved window (which is likely at the end due to update_window_state)
      if let Some(pos) =
        windows.iter().position(|w| w.id() == moved_window.id())
      {
        windows.remove(pos);
      }

      // Find the target window's index
      if let Some(target_idx) =
        windows.iter().position(|w| w.id() == target_window.id())
      {
        let insert_idx = if insert_after {
          target_idx + 1
        } else {
          target_idx
        };

        // Insert the moved window at the new position
        if insert_idx <= windows.len() {
          windows.insert(
            insert_idx,
            moved_window
              .as_tiling_window()
              .context("Moved window is not tiling")?
              .clone(),
          );
        } else {
          windows.push(
            moved_window
              .as_tiling_window()
              .context("Moved window is not tiling")?
              .clone(),
          );
        }

        // Rebuild the spiral with the new order
        rebuild_spiral_layout(&workspace, &windows)?;

        // Queue redraws
        state
          .pending_sync
          .queue_containers_to_redraw(workspace.tiling_children());
      }
    }
  }

  Ok(())
}

/// Represents where the window was dropped over another.
#[derive(Debug, Clone, PartialEq)]
enum DropPosition {
  Top,
  Bottom,
  Left,
  Right,
}

/// Gets the drop position for a window based on the mouse position.
///
/// This approach divides the window rect into an "X", creating four
/// triangular quadrants, to determine which side the cursor is closest to.
fn drop_position(mouse_pos: &Point, rect: &Rect) -> DropPosition {
  let delta_x = mouse_pos.x - rect.center_point().x;
  let delta_y = mouse_pos.y - rect.center_point().y;

  if delta_x.abs() > delta_y.abs() {
    // Window is in the left or right triangle.
    if delta_x > 0 {
      DropPosition::Right
    } else {
      DropPosition::Left
    }
  } else {
    // Window is in the top or bottom triangle.
    if delta_y > 0 {
      DropPosition::Bottom
    } else {
      DropPosition::Top
    }
  }
}
