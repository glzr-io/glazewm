use anyhow::Context;
use tracing::info;

use crate::{
  common::{
    platform::{NativeWindow, Platform},
    LengthValue, Point, Rect, TilingDirection,
  },
  containers::{
    commands::{move_container_within_tree, wrap_in_split_container},
    traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
    DirectionContainer, SplitContainer, TilingContainer, WindowContainer,
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
        // Don't update state on resize events if the WM is paused.
        if state.is_paused {
          return Ok(());
        }

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
  let workspace = moved_window.workspace().context("No workspace.")?;

  // Get the workspace, split containers, and other windows under the
  // dragged window.
  let containers_at_pos = state
    .containers_at_point(workspace.clone().into(), &mouse_pos)
    .into_iter()
    .filter(|container| container.id() != moved_window.id());

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

  let nearest_container = target_parent
    .children()
    .into_iter()
    .filter_map(|container| container.as_tiling_container().ok())
    .try_fold(None, |acc: Option<TilingContainer>, container| match acc {
      Some(acc) => {
        let is_nearer = acc.to_rect()?.distance_to_point(&mouse_pos)
          < container.to_rect()?.distance_to_point(&mouse_pos);

        anyhow::Ok(Some(if is_nearer { acc } else { container }))
      }
      None => Ok(Some(container)),
    })?
    .context("No nearest container.")?;

  let tiling_direction = target_parent.tiling_direction();
  let drop_position =
    drop_position(&mouse_pos, &nearest_container.to_rect()?);

  let moved_window = update_window_state(
    moved_window.clone().into(),
    WindowState::Tiling,
    state,
    config,
  )?;

  let should_split = nearest_container.is_tiling_window()
    && match tiling_direction {
      TilingDirection::Horizontal => {
        drop_position == DropPosition::Top
          || drop_position == DropPosition::Bottom
      }
      TilingDirection::Vertical => {
        drop_position == DropPosition::Left
          || drop_position == DropPosition::Right
      }
    };

  if should_split {
    let split_container = SplitContainer::new(
      tiling_direction.inverse(),
      config.value.gaps.clone(),
    );

    wrap_in_split_container(
      split_container.clone(),
      target_parent.clone().into(),
      vec![nearest_container],
    )?;

    let target_index = match drop_position {
      DropPosition::Top | DropPosition::Left => 0,
      _ => 1,
    };

    move_container_within_tree(
      moved_window.clone().into(),
      split_container.into(),
      target_index,
      state,
    )?;
  } else {
    let target_index = match drop_position {
      DropPosition::Top | DropPosition::Left => nearest_container.index(),
      _ => nearest_container.index() + 1,
    };

    move_container_within_tree(
      moved_window.clone().into(),
      target_parent.clone().into(),
      target_index,
      state,
    )?;
  }

  state
    .pending_sync
    .containers_to_redraw
    .push(target_parent.into());

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

  match delta_x.abs() > delta_y.abs() {
    // Window is in the left or right triangle.
    true => match delta_x > 0 {
      true => DropPosition::Right,
      false => DropPosition::Left,
    },
    // Window is in the top or bottom triangle.
    false => match delta_y > 0 {
      true => DropPosition::Bottom,
      false => DropPosition::Top,
    },
  }
}
