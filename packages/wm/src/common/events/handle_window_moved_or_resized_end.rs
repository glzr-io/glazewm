use anyhow::Context;
use tracing::{debug, info};

use crate::{
  common::{
    platform::{NativeWindow, Platform},
    LengthValue, Point, TilingDirection,
  },
  containers::{
    commands::{
      attach_container, detach_container, move_container_within_tree,
      wrap_in_split_container,
    },
    traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
    Container, SplitContainer, TilingContainer, WindowContainer,
  },
  user_config::UserConfig,
  windows::{
    active_drag::ActiveDragOperation,
    commands::{resize_window, update_window_state},
    traits::WindowGetters,
    ActiveDrag, NonTilingWindow, TilingWindow, WindowState,
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

    let parent = window.parent().context("No parent.")?;

    // Snap window to its original position if it's the only window in the
    // workspace.
    if parent.is_workspace() && window.tiling_siblings().count() == 0 {
      state.pending_sync.containers_to_redraw.push(window.into());
      return Ok(());
    }

    let new_rect = window.native().refresh_frame_position()?;
    let old_rect = window.to_rect()?;

    let width_delta = new_rect.width() - old_rect.width();
    let height_delta = new_rect.height() - old_rect.height();

    if let WindowContainer::NonTilingWindow(window) = window {
      let has_window_moved = matches!((width_delta, height_delta), (0, 0));

      if has_window_moved {
        window_moved_end(window, state, config)?;
      }
    } else if let WindowContainer::TilingWindow(window) = window {
      window_resized_end(window, state, width_delta, height_delta)?;
    }
  }

  Ok(())
}

/// Handles window resize events
fn window_resized_end(
  window: TilingWindow,
  state: &mut WmState,
  width_delta: i32,
  height_delta: i32,
) -> anyhow::Result<()> {
  info!("Tiling window resized");
  resize_window(
    window.clone().into(),
    Some(LengthValue::from_px(width_delta)),
    Some(LengthValue::from_px(height_delta)),
    state,
  )
}

/// Handles window move events
fn window_moved_end(
  moved_window: NonTilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // We continue only if it's a temporary Floating window and if the window
  // got moved and not resized
  let mut active_drag = moved_window.active_drag();
  if active_drag.is_from_tiling == false
    || !matches!(active_drag.operation, Some(ActiveDragOperation::Moving))
  {
    active_drag.operation = None;
    moved_window.set_active_drag(active_drag);
    return Ok(());
  }
  info!("Tiling window drag end event");

  let mouse_position = Platform::mouse_position()?;

  let window_under_cursor = match get_tiling_window_at_mouse_pos(
    &moved_window,
    &mouse_position,
    state,
  ) {
    Some(value) => value,
    None => {
      update_window_state(
        moved_window.as_window_container().unwrap(),
        WindowState::Tiling,
        state,
        config,
      )?;
      return Ok(());
    }
  };

  debug!(
    "Moved window: {:?} \n Target window: {:?}",
    moved_window.native().process_name(),
    window_under_cursor.native().process_name(),
  );

  let tiling_direction = get_split_direction(&window_under_cursor)?;
  let new_window_position = get_drop_position(
    &mouse_position,
    &window_under_cursor,
    &tiling_direction,
  )?;

  let parent = window_under_cursor
    .direction_container()
    .context("The window has no direction container")?;
  let parent_tiling_direction: TilingDirection = parent.tiling_direction();

  move_window_to_target(
    state,
    config,
    moved_window.clone(),
    window_under_cursor.clone(),
    &parent.into(),
    parent_tiling_direction,
    tiling_direction,
    new_window_position,
  )?;
  moved_window.set_active_drag(ActiveDrag::default());

  state.pending_sync.containers_to_redraw.push(
    window_under_cursor
      .workspace()
      .context("No workspace")?
      .into(),
  );

  Ok(())
}

/// Return the window under the mouse position excluding the dragged window
fn get_tiling_window_at_mouse_pos(
  exclude_window: &NonTilingWindow,
  mouse_position: &Point,
  state: &WmState,
) -> Option<TilingWindow> {
  let children_at_mouse_position: Vec<TilingWindow> = state
    .window_containers_at_position(mouse_position)
    .into_iter()
    .filter_map(|container| match container {
      WindowContainer::TilingWindow(tiling) => Some(tiling),
      _ => None,
    })
    .filter(|window: &TilingWindow| window.id() != exclude_window.id())
    .collect();

  children_at_mouse_position.into_iter().next()
}

fn move_window_to_target(
  state: &mut WmState,
  config: &UserConfig,
  moved_window: NonTilingWindow,
  target_window: TilingWindow,
  target_window_parent: &Container,
  current_tiling_direction: TilingDirection,
  new_tiling_direction: TilingDirection,
  drop_position: DropPosition,
) -> anyhow::Result<()> {
  update_window_state(
    moved_window.as_window_container().unwrap(),
    WindowState::Tiling,
    state,
    config,
  )?;

  let moved_window = state
    .windows()
    .iter()
    .find(|w| w.id() == moved_window.id())
    .context("couldn't find the new tiled window")?
    .as_tiling_window()
    .context("window is not a tiled window")?
    .clone();

  // TODO: We can optimize that by not detaching and attaching the window
  // Little trick to get the right index
  detach_container(Container::TilingWindow(moved_window.clone()))?;
  let target_window_index = target_window.index();
  attach_container(
    &Container::TilingWindow(moved_window.clone()),
    target_window_parent,
    None,
  )?;

  let target_index = match drop_position {
    DropPosition::Start => target_window_index,
    DropPosition::End => target_window_index + 1,
  };

  match (new_tiling_direction, current_tiling_direction) {
    (TilingDirection::Horizontal, TilingDirection::Horizontal)
    | (TilingDirection::Vertical, TilingDirection::Vertical) => {
      move_container_within_tree(
        Container::TilingWindow(moved_window.clone()),
        target_window_parent.clone(),
        target_index,
        state,
      )?;
    }
    (TilingDirection::Horizontal, TilingDirection::Vertical) => {
      create_split_container(
        TilingDirection::Horizontal,
        config,
        moved_window,
        target_window,
        drop_position,
        &target_window_parent,
      )?;
    }
    (TilingDirection::Vertical, TilingDirection::Horizontal) => {
      create_split_container(
        TilingDirection::Vertical,
        config,
        moved_window,
        target_window,
        drop_position,
        &target_window_parent,
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
    DropPosition::Start => 0,
    DropPosition::End => 1,
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
/// [DropPosition::Start] can either be the top or left side.
/// [DropPosition::Stop] can either be bottom or right side.
#[derive(Debug)]
enum DropPosition {
  Start,
  End,
}

/// Determines the drop position for a window based on the mouse position
/// and tiling direction.
///
/// This function calculates whether a window should be dropped at the
/// start or end of a tiling layout, depending on the mouse position
/// relative to the middle of the target window.
fn get_drop_position(
  mouse_position: &Point,
  window: &TilingWindow,
  tiling_direction: &TilingDirection,
) -> anyhow::Result<DropPosition> {
  let rect = window.to_rect()?;

  match tiling_direction {
    TilingDirection::Vertical => {
      let middle = rect.top + (rect.height() / 2);
      if mouse_position.y < middle {
        Ok(DropPosition::Start)
      } else {
        Ok(DropPosition::End)
      }
    }
    TilingDirection::Horizontal => {
      let middle = rect.left + (rect.width() / 2);
      if mouse_position.x < middle {
        Ok(DropPosition::Start)
      } else {
        Ok(DropPosition::End)
      }
    }
  }
}

/// Determines the optimal split direction for a given window.
///
/// This function decides whether a window should be split vertically or
/// horizontally based on its current dimensions.
fn get_split_direction(
  window: &TilingWindow,
) -> anyhow::Result<TilingDirection> {
  let rect = window.to_rect()?;

  if rect.height() > rect.width() {
    Ok(TilingDirection::Vertical)
  } else {
    Ok(TilingDirection::Horizontal)
  }
}
