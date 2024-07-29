use anyhow::Context;
use tracing::info;
use crate::{
  common::{platform::Platform, Point, TilingDirection},
  containers::{
    commands::{attach_container, move_container_within_tree},
    traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
    Container, SplitContainer,
  },
  user_config::UserConfig,
  windows::{
    commands::update_window_state, traits::WindowGetters, NonTilingWindow,
    TilingWindow, WindowState,
  },
  wm_state::WmState,
};
use crate::containers::commands::detach_container;
use crate::user_config::FloatingStateConfig;

/// Handles window move events
pub fn window_moved_end(
  moved_window: NonTilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // We continue only if it's a temporary Floating window
  if matches!(moved_window.state(), WindowState::Floating(FloatingStateConfig{is_tiling_drag: false,..})){
    return Ok(());
  }
  info!("Tiling window drag end");

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

  Ok(())
}

fn move_window_to_target(
  state: &mut WmState,
  config: &UserConfig,
  moved_window: NonTilingWindow,
  target_window: TilingWindow,
  target_window_parent: &Container,
  current_tiling_direction: TilingDirection,
  new_tiling_direction: TilingDirection,
  new_window_position: DropPosition,
) -> anyhow::Result<()> {

  // TODO: We can optimize that for sure by not detaching and attaching the window
  // Little trick to get the right index
  detach_container(Container::NonTilingWindow(moved_window.clone()))?;
  let target_window_index = target_window.index();
  attach_container(&Container::NonTilingWindow(moved_window.clone()), target_window_parent, None)?;

  let target_index = match new_window_position {
    DropPosition::Start => Some(target_window_index),
    DropPosition::End => Some(target_window_index + 1),
  };

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


  match (new_tiling_direction, current_tiling_direction) {
    (TilingDirection::Horizontal, TilingDirection::Horizontal)
    | (TilingDirection::Vertical, TilingDirection::Vertical) => {
      info!("Target window index {}", target_window_index);

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
        new_window_position,
        &target_window_parent,
        target_index,
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
  split_container_index: Option<usize>,
) -> anyhow::Result<()> {
  let target_index_inside_split_container = match dropped_position {
    DropPosition::Start => Some(0),
    DropPosition::End => Some(1),
  };

  let split_container = Container::Split(SplitContainer::new(
    tiling_direction,
    config.value.gaps.inner_gap.clone(),
  ));
  attach_container(&split_container, &parent, split_container_index)?;

  move_container_within_tree(
    Container::TilingWindow(target_window),
    split_container.clone(),
    None,
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
