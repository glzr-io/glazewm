use anyhow::Context;
use tracing::{debug, info};

use crate::{
  common::{platform::Platform, Point, TilingDirection},
  containers::{
    commands::{
      attach_container, detach_container, get_containers_at_position,
      move_container_within_tree,
    },
    traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
    Container, SplitContainer, WindowContainer,
  },
  user_config::{FloatingStateConfig, UserConfig},
  windows::{
    commands::update_window_state,
    traits::WindowGetters,
    window_operation::{Operation, WindowOperation},
    NonTilingWindow, TilingWindow, WindowState,
  },
  wm_state::WmState,
};

/// Handles window move events
pub fn window_moved_end(
  moved_window: NonTilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // We continue only if it's a temporary Floating window and if the window
  // got moved and not resized
  if matches!(
    moved_window.state(),
    WindowState::Floating(FloatingStateConfig {
      is_tiling_drag: false,
      ..
    })
  ) || moved_window.window_operation().operation != Operation::Moving
  {
    moved_window.set_window_operation(WindowOperation::default());
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

  let parent = window_under_cursor.parent().unwrap();

  match &parent {
    // If the parent is a workspace we only need to add the window to it or
    // to create a Vertical split container
    Container::Workspace(_) => {
      let current_tiling_direction = TilingDirection::Horizontal;
      move_window_to_target(
        state,
        config,
        moved_window.clone(),
        window_under_cursor.clone(),
        &parent,
        current_tiling_direction,
        tiling_direction,
        new_window_position,
      )?;
    }
    // If the parent is a split we need to check the current split
    // direction add the window to it or create a [Vertical/Horizontal]
    // split container
    Container::Split(split) => {
      let current_tiling_direction = split.tiling_direction();
      move_window_to_target(
        state,
        config,
        moved_window.clone(),
        window_under_cursor.clone(),
        &parent,
        current_tiling_direction,
        tiling_direction,
        new_window_position,
      )?;
    }
    _ => {}
  }
  moved_window.set_window_operation(WindowOperation::default());
  state
    .pending_sync
    .containers_to_redraw
    .push(Container::Workspace(
      window_under_cursor.workspace().unwrap(),
    ));

  Ok(())
}

/// Return the window under the mouse position excluding the dragged window
fn get_tiling_window_at_mouse_pos(
  exclude_window: &NonTilingWindow,
  mouse_position: &Point,
  state: &WmState,
) -> Option<TilingWindow> {
  let children_at_mouse_position: Vec<TilingWindow> =
    get_containers_at_position(state, mouse_position)
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
        state,
        config,
        moved_window,
        target_window,
        drop_position,
        &target_window_parent,
        target_index,
      )?;
    }
    (TilingDirection::Vertical, TilingDirection::Horizontal) => {
      create_split_container(
        TilingDirection::Vertical,
        state,
        config,
        moved_window,
        target_window,
        drop_position,
        &target_window_parent,
        target_index,
      )?;
    }
  }

  Ok(())
}

/// Creates a split container and moves the target window and the moved
/// window inside at the dropped position
fn create_split_container(
  tiling_direction: TilingDirection,
  state: &mut WmState,
  config: &UserConfig,
  moved_window: TilingWindow,
  target_window: TilingWindow,
  dropped_position: DropPosition,
  parent: &Container,
  split_container_index: usize,
) -> anyhow::Result<()> {
  let target_index_inside_split_container = match dropped_position {
    DropPosition::Start => 0,
    DropPosition::End => 1,
  };

  let split_container = Container::Split(SplitContainer::new(
    tiling_direction,
    config.value.gaps.inner_gap.clone(),
  ));
  attach_container(
    &split_container,
    &parent,
    Some(split_container_index),
  )?;

  move_container_within_tree(
    Container::TilingWindow(target_window),
    split_container.clone(),
    0,
    state,
  )?;
  move_container_within_tree(
    Container::TilingWindow(moved_window),
    split_container,
    target_index_inside_split_container,
    state,
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
