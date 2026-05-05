use anyhow::Context;
use wm_common::{TilingDirection, WindowState};
use wm_platform::Direction;

use super::set_focused_descendant;
use crate::{
  commands::workspace::focus_workspace,
  models::{Container, TilingContainer, WorkspaceTarget},
  traits::{CommonGetters, TilingDirectionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn focus_in_direction_with_wrap(
  origin_container: &Container,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focus_target = match origin_container {
    Container::TilingWindow(_) => {
      // Try to find the target in the current workspace or across
      // monitors.
      tiling_focus_target(origin_container, direction)?.map_or_else(
        || workspace_focus_target(origin_container, direction, state),
        |container| Ok(Some(container)),
      )?
    }
    Container::NonTilingWindow(ref non_tiling_window) => {
      match non_tiling_window.state() {
        WindowState::Floating(_) => {
          floating_focus_target(origin_container, direction)
        }
        WindowState::Fullscreen(_) => {
          workspace_focus_target(origin_container, direction, state)?
        }
        _ => None,
      }
    }
    Container::Workspace(_) => {
      workspace_focus_target(origin_container, direction, state)?
    }
    _ => None,
  };

  if let Some(focus_target) = focus_target {
    // Once the target is found, switch focus directly.
    set_focused_descendant(&focus_target, None);
    state.pending_sync.queue_focus_change().queue_cursor_jump();
  } else {
    // Wrap Logic: "Target not found" indicates that the display boundary
    // has been reached. Switch to adjacent workspace
    let workspace_target = match direction {
      Direction::Left | Direction::Up => WorkspaceTarget::Previous,
      Direction::Right | Direction::Down => WorkspaceTarget::Next,
    };

    focus_workspace(workspace_target, state, config)?;

    // Set the focus on the outermost container in the opposite direction
    // (i.e., find the left edge in the Right direction).
    if let Some(new_ws) =
      state.focused_container().and_then(|c| c.workspace())
    {
      let edge_container =
        new_ws.descendant_in_direction(&direction.inverse());
      let focus_container = edge_container
        .map(Into::into)
        .unwrap_or_else(|| new_ws.clone().into());

      set_focused_descendant(&focus_container, None);
      state.pending_sync.queue_focus_change().queue_cursor_jump();
    }
  }

  Ok(())
}

fn floating_focus_target(
  origin_container: &Container,
  direction: &Direction,
) -> Option<Container> {
  let is_floating = |sibling: &Container| {
    sibling.as_non_tiling_window().is_some_and(|window| {
      matches!(window.state(), WindowState::Floating(_))
    })
  };

  let mut floating_siblings =
    origin_container.siblings().filter(is_floating);

  match direction {
    Direction::Left => origin_container
      .next_siblings()
      .find(is_floating)
      .or_else(|| floating_siblings.last()),
    Direction::Right => origin_container
      .prev_siblings()
      .find(is_floating)
      .or_else(|| floating_siblings.next()),
    _ => None,
  }
}

fn tiling_focus_target(
  origin_container: &Container,
  direction: &Direction,
) -> anyhow::Result<Option<Container>> {
  let tiling_direction = TilingDirection::from_direction(direction);
  let mut origin_or_ancestor = origin_container.clone();

  while !origin_or_ancestor.is_workspace() {
    let parent = origin_or_ancestor
      .parent()
      .and_then(|parent| parent.as_direction_container().ok())
      .context("No direction container.")?;

    if parent.tiling_direction() != tiling_direction {
      origin_or_ancestor = parent.into();
      continue;
    }

    let focus_target = match direction {
      Direction::Up | Direction::Left => origin_or_ancestor
        .prev_siblings()
        .find_map(|c| c.as_tiling_container().ok()),
      _ => origin_or_ancestor
        .next_siblings()
        .find_map(|c| c.as_tiling_container().ok()),
    };

    match focus_target {
      Some(target) => {
        return Ok(match target {
          TilingContainer::TilingWindow(_) => Some(target.into()),
          TilingContainer::Split(split) => split
            .descendant_in_direction(&direction.inverse())
            .map(Into::into),
        });
      }
      None => origin_or_ancestor = parent.into(),
    }
  }

  Ok(None)
}

fn workspace_focus_target(
  origin_container: &Container,
  direction: &Direction,
  state: &WmState,
) -> anyhow::Result<Option<Container>> {
  let monitor = origin_container.monitor().context("No monitor.")?;

  let target_workspace = state
    .monitor_in_direction(&monitor, direction)?
    .and_then(|monitor| monitor.displayed_workspace());

  let focused_fullscreen = target_workspace
    .as_ref()
    .and_then(|workspace| workspace.descendant_focus_order().next())
    .filter(|focused| match focused {
      Container::NonTilingWindow(window) => {
        matches!(window.state(), WindowState::Fullscreen(_))
      }
      _ => false,
    });

  let focus_target = focused_fullscreen
    .or_else(|| {
      target_workspace.as_ref().and_then(|workspace| {
        workspace
          .descendant_in_direction(&direction.inverse())
          .map(Into::into)
      })
    })
    .or(target_workspace.map(Into::into));

  Ok(focus_target)
}
