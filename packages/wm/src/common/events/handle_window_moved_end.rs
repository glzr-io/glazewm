use std::{collections::VecDeque, io::Split};

use anyhow::Context;
use tracing::{debug, info};
use windows::Win32::{Foundation, UI::WindowsAndMessaging::GetCursorPos};

use crate::{
  common::{
    commands::platform_sync,
    platform::{MouseMoveEvent, NativeWindow, Platform},
    LengthValue, Point, Rect, TilingDirection,
  },
  containers::{
    commands::{
      attach_container, detach_container, move_container_within_tree,
    },
    traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
    Container, SplitContainer, WindowContainer,
  },
  user_config::UserConfig,
  windows::{
    commands::resize_window, traits::WindowGetters, NonTilingWindow,
    TilingWindow,
  },
  wm_event::WmEvent,
  wm_state::WmState,
};

/// Handles window move events
pub fn window_moved_end(
  moved_window: TilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Tiling window moved");

  let workspace = moved_window
    .workspace()
    .context("Couldn't find a workspace")?;

  let mouse_position = Platform::mouse_position()?;

  let children_at_mouse_position: Vec<_> = workspace
    .descendants()
    .filter_map(|container| match container {
      Container::TilingWindow(tiling) => Some(tiling),
      _ => None,
    })
    .filter(|c| {
      let frame = c.to_rect();
      info!("{:?}", frame);
      info!("{:?}", mouse_position);
      frame.unwrap().contains_point(&mouse_position)
    })
    .filter(|window| window.id() != moved_window.id())
    .collect();

  if children_at_mouse_position.is_empty() {
    return Ok(());
  }

  let window_under_cursor =
    children_at_mouse_position.into_iter().next().unwrap();

  info!(
    "Swapping {} with {}",
    moved_window
      .native()
      .process_name()
      .unwrap_or("".to_string()),
    window_under_cursor
      .native()
      .process_name()
      .unwrap_or("".to_string())
  );

  let tiling_direction = get_split_direction(&window_under_cursor)?;
  let new_window_position = get_new_window_position(
    &mouse_position,
    &window_under_cursor,
    &tiling_direction,
  )?;

  info!("{:?} {:?}", tiling_direction, new_window_position);

  let parent = window_under_cursor.parent().unwrap();

  match &parent {
    // If the parent is a workspace we only need to add the window to it or
    // to create a VerticalSplitDirection
    Container::Workspace(_) => {
      let current_tiling_direction = TilingDirection::Horizontal;
      move_window_to_target(
        state,
        config,
        moved_window,
        window_under_cursor,
        &parent,
        current_tiling_direction,
        tiling_direction,
        new_window_position,
      )?;
    }
    // If the parent is a split we need to check the current split
    // direction add the window to it or create a VerticalSplitDirection
    Container::Split(split) => {
      let current_tiling_direction = split.tiling_direction();
      move_window_to_target(
        state,
        config,
        moved_window,
        window_under_cursor,
        &parent,
        current_tiling_direction,
        tiling_direction,
        new_window_position,
      )?;
    }
    _ => {}
  }

  state.pending_sync.containers_to_redraw.push(parent);

  // info!("{:#?}", workspace);

  Ok(())
}

fn move_window_to_target(
  state: &mut WmState,
  config: &UserConfig,
  moved_window: TilingWindow,
  target_window: TilingWindow,
  target_window_parent: &Container,
  current_tiling_direction: TilingDirection,
  new_tiling_direction: TilingDirection,
  new_window_position: DropPosition,
) -> anyhow::Result<()> {
  let target_window_index = target_window_parent
    .children()
    .iter()
    .position(|c| {
      if let Some(tiling_window) = c.as_tiling_window() {
        tiling_window.id() == target_window.id()
      } else {
        false
      }
    })
    .context("Window index not found")?;

  match (new_tiling_direction, current_tiling_direction) {
    (TilingDirection::Horizontal, TilingDirection::Horizontal)
    | (TilingDirection::Vertical, TilingDirection::Vertical) => {
      let target_index = match new_window_position {
        DropPosition::Start => Some(target_window_index),
        DropPosition::End => None,
      };

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
        new_window_position,
        &target_window_parent,
      )?;
    }
    (TilingDirection::Vertical, TilingDirection::Horizontal) => {
      create_split_container(
        TilingDirection::Vertical,
        state,
        config,
        moved_window,
        target_window,
        new_window_position,
        &target_window_parent,
      )?;
    }
  }

  Ok(())
}

fn create_split_container(
  tiling_direction: TilingDirection,
  state: &mut WmState,
  config: &UserConfig,
  moved_window: TilingWindow,
  target_window: TilingWindow,
  dropped_position: DropPosition,
  parent: &Container,
) -> anyhow::Result<()> {
  let target_index = match dropped_position {
    DropPosition::Start => Some(0),
    DropPosition::End => None,
  };

  let split_container = Container::Split(SplitContainer::new(
    tiling_direction,
    config.value.gaps.inner_gap.clone(),
  ));
  attach_container(&split_container, &parent, None)?;

  move_container_within_tree(
    Container::TilingWindow(target_window),
    split_container.clone(),
    None,
    state,
  )?;
  move_container_within_tree(
    Container::TilingWindow(moved_window),
    split_container,
    target_index,
    state,
  )?;
  Ok(())
}

#[derive(Debug)]
enum DropPosition {
  Start,
  End,
}

fn get_new_window_position(
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
