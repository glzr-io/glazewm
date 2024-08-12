use anyhow::Context;
use tracing::{debug, info};

use crate::{
  common::{
    platform::{NativeWindow, Platform},
    LengthValue, Point, Rect, TilingDirection,
  },
  containers::{
    commands::{
      attach_container, detach_container, move_container_within_tree,
      wrap_in_split_container,
    },
    traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
    Container, SplitContainer, TilingContainer, WindowContainer,
  },
  try_warn,
  user_config::UserConfig,
  windows::{
    commands::{resize_window, update_window_state},
    traits::WindowGetters,
    ActiveDragOperation, NonTilingWindow, TilingWindow, WindowState,
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

  let window_under_cursor =
    tiling_window_at_mouse_pos(&moved_window, &mouse_pos, state);

  if let None = window_under_cursor {
    update_window_state(
      moved_window.clone().into(),
      WindowState::Tiling,
      state,
      config,
    )?;

    return Ok(());
  }

  let window_under_cursor = window_under_cursor.unwrap();
  let drop_position =
    determine_drop_position(&mouse_pos, &window_under_cursor.to_rect()?);
  
  move_window_to_target(
    state,
    config,
    moved_window.clone(),
    window_under_cursor.clone(),
    drop_position,
  )?;

  state.pending_sync.containers_to_redraw.push(
    window_under_cursor
      .workspace()
      .context("No workspace")?
      .into(),
  );

  Ok(())
}

/// Gets the window under the mouse position excluding the dragged window.
fn tiling_window_at_mouse_pos(
  exclude_window: &NonTilingWindow,
  mouse_position: &Point,
  state: &WmState,
) -> Option<TilingWindow> {
  state
    .containers_at_point(mouse_position)
    .into_iter()
    .filter_map(|container| match container {
      Container::TilingWindow(t) => Some(WindowContainer::TilingWindow(t)),
      Container::NonTilingWindow(nt) => {
        Some(WindowContainer::NonTilingWindow(nt))
      }
      _ => None,
    })
    .filter_map(|window| window.as_tiling_window().cloned())
    .filter(|window| window.id() != exclude_window.id())
    .next()
}

fn move_window_to_target(
  state: &mut WmState,
  config: &UserConfig,
  moved_window: NonTilingWindow,
  target_window: TilingWindow,
  drop_position: DropPosition,
) -> anyhow::Result<()> {
  update_window_state(
    moved_window.as_window_container().unwrap(),
    WindowState::Tiling,
    state,
    config,
  )?;

  let target_window_parent = target_window
    .direction_container()
    .context("The window has no direction container")?;
  let target_window_parent_tiling_direction: TilingDirection =
    target_window_parent.tiling_direction();

  // Getting the new window handler after changing it state
  let moved_window = state
    .windows()
    .iter()
    .find(|w| w.id() == moved_window.id())
    .context("couldn't find the new tiled window")?
    .as_tiling_window()
    .context("window is not a tiled window")?
    .clone();

  // TODO: We can optimize that by not detaching and attaching the window
  // Detaching and reattaching helps to get the right index in certain
  // condition
  detach_container(Container::TilingWindow(moved_window.clone()))?;
  let target_window_index = target_window.index();
  attach_container(
    &Container::TilingWindow(moved_window.clone()),
    &target_window_parent.as_container(),
    None,
  )?;

  let target_index = match drop_position {
    DropPosition::Top | DropPosition::Left => target_window_index,
    DropPosition::Bottom | DropPosition::Right => target_window_index + 1,
  };

  match (drop_position.clone(), target_window_parent_tiling_direction) {
    (DropPosition::Right, TilingDirection::Horizontal)
    | (DropPosition::Left, TilingDirection::Horizontal)
    | (DropPosition::Top, TilingDirection::Vertical)
    | (DropPosition::Bottom, TilingDirection::Vertical) => {
      move_container_within_tree(
        Container::TilingWindow(moved_window.clone()),
        target_window_parent.as_container(),
        target_index,
        state,
      )?;
    }
    (DropPosition::Left, TilingDirection::Vertical)
    | (DropPosition::Right, TilingDirection::Vertical) => {
      create_split_container(
        TilingDirection::Horizontal,
        config,
        moved_window,
        target_window,
        drop_position,
        &target_window_parent.as_container(),
      )?;
    }
    (DropPosition::Top, TilingDirection::Horizontal)
    | (DropPosition::Bottom, TilingDirection::Horizontal) => {
      create_split_container(
        TilingDirection::Vertical,
        config,
        moved_window,
        target_window,
        drop_position,
        &target_window_parent.as_container(),
      )?;
    }
  }

  Ok(())
}

/// Creates a split container and moves the target window and the moved
/// window inside at the dropped position
fn create_split_container(
  tiling_direction: TilingDirection,
  config: &UserConfig,
  moved_window: TilingWindow,
  target_window: TilingWindow,
  dropped_position: DropPosition,
  parent: &Container,
) -> anyhow::Result<()> {
  let target_index_inside_split_container = match dropped_position {
    DropPosition::Top | DropPosition::Left => 0,
    DropPosition::Bottom | DropPosition::Right => 1,
  };

  let split_container = SplitContainer::new(
    tiling_direction,
    config.value.gaps.inner_gap.clone(),
  );

  let mut split_container_children =
    vec![TilingContainer::TilingWindow(target_window)];

  split_container_children.insert(
    target_index_inside_split_container,
    TilingContainer::TilingWindow(moved_window),
  );

  wrap_in_split_container(
    split_container,
    parent.clone(),
    split_container_children,
  )?;
  Ok(())
}

/// Represents where the window was dropped over another one.
/// It depends on the tiling direction.
///
/// [DropPosition::Top] can either be the top or left side.
/// [DropPosition::Stop] can either be bottom or right side.
#[derive(Debug, Clone)]
enum DropPosition {
  Top,
  Bottom,
  Left,
  Right,
}

/// Determines the drop position for a window based on the mouse position
/// in a four-triangle pattern (X pattern).
///
/// This function calculates whether the mouse is in the top, bottom,
/// left, or right triangular region of the window.
fn determine_drop_position(
  mouse_position: &Point,
  frame: &Rect,
) -> DropPosition {
  let x = mouse_position.x;
  let y = mouse_position.y;

  // Calculate the center of the frame
  let center_x = (frame.left + frame.right) / 2;
  let center_y = (frame.top + frame.bottom) / 2;

  // Calculate the slopes of the diagonals
  let diag1_slope =
    (frame.bottom - frame.top) as f32 / (frame.right - frame.left) as f32; // positive slope
  let diag2_slope = -diag1_slope; // negative slope

  // Calculate the y values on the diagonals for the given x position
  let diag1_y_at_x = center_y as f32 + diag1_slope * (x - center_x) as f32;
  let diag2_y_at_x = center_y as f32 + diag2_slope * (x - center_x) as f32;

  // Determine the position based on the region
  if (y as f32) < diag1_y_at_x && (y as f32) < diag2_y_at_x {
    DropPosition::Top
  } else if (y as f32) > diag1_y_at_x && (y as f32) > diag2_y_at_x {
    DropPosition::Bottom
  } else if (y as f32) > diag1_y_at_x && (y as f32) < diag2_y_at_x {
    DropPosition::Left
  } else {
    DropPosition::Right
  }
}
