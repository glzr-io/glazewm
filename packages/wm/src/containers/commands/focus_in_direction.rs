use anyhow::Context;

use super::set_focused_descendant;
use crate::{
  common::{Direction, TilingDirection},
  containers::{
    traits::{CommonGetters, TilingDirectionGetters},
    Container, TilingContainer,
  },
  user_config::{CursorJumpTrigger, UserConfig},
  windows::{traits::WindowGetters, WindowState},
  wm_state::WmState,
};

pub fn focus_in_direction(
  origin_container: Container,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focus_target = match origin_container {
    Container::TilingWindow(_) => {
      // If a suitable focus target isn't found in the current workspace,
      // attempt to find a workspace in the given direction.
      tiling_focus_target(origin_container.clone(), &direction)?
        .map_or_else(
          || workspace_focus_target(origin_container, &direction, state),
          |container| Ok(Some(container)),
        )?
    }
    Container::NonTilingWindow(ref non_tiling_window) => {
      match non_tiling_window.state() {
        WindowState::Floating(_) => {
          floating_focus_target(origin_container, &direction)
        }
        WindowState::Fullscreen(_) => {
          workspace_focus_target(origin_container, &direction, state)?
        }
        _ => None,
      }
    }
    Container::Workspace(_) => {
      workspace_focus_target(origin_container, &direction, state)?
    }
    _ => None,
  };

  // Set focus to the target container.
  if let Some(focus_target) = focus_target {
    set_focused_descendant(focus_target.clone(), None);
    state.pending_sync.focus_change = true;

    // Jump cursor to the focus target if enabled.
    if config.value.general.cursor_jump.enabled {
      match config.value.general.cursor_jump.trigger {
        CursorJumpTrigger::WindowFocus => {
          state.pending_sync.cursor_container = Some(focus_target);
        }
        CursorJumpTrigger::MonitorFocus => {
          let monitor = focus_target.monitor().context("No monitor.")?;
          state.pending_sync.cursor_container = Some(monitor.into());
        }
      }
    }
  }

  Ok(())
}

fn floating_focus_target(
  origin_container: Container,
  direction: &Direction,
) -> Option<Container> {
  let is_floating = |sibling: &Container| {
    sibling.as_non_tiling_window().map_or(false, |window| {
      matches!(window.state(), WindowState::Floating(_))
    })
  };

  let mut floating_siblings =
    origin_container.siblings().filter(is_floating);

  // Wrap if next/previous floating window is not found.
  match direction {
    Direction::Left => origin_container
      .next_siblings()
      .find(is_floating)
      .or_else(|| floating_siblings.last()),
    Direction::Right => origin_container
      .prev_siblings()
      .find(is_floating)
      .or_else(|| floating_siblings.next()),
    // Cannot focus vertically from a floating window.
    _ => None,
  }
}

/// Gets a focus target within the current workspace. Traverse upwards from
/// the origin container to find an adjacent container that can be focused.
fn tiling_focus_target(
  origin_container: Container,
  direction: &Direction,
) -> anyhow::Result<Option<Container>> {
  let tiling_direction = TilingDirection::from_direction(direction);
  let mut origin_or_ancestor = origin_container.clone();

  // Traverse upwards from the focused container. Stop searching when a
  // workspace is encountered.
  while !origin_or_ancestor.is_workspace() {
    let parent = origin_or_ancestor
      .parent()
      .and_then(|parent| parent.as_direction_container().ok())
      .context("No direction container.")?;

    // Skip if the tiling direction doesn't match.
    if parent.tiling_direction() != tiling_direction {
      origin_or_ancestor = parent.into();
      continue;
    }

    // Get the next/prev tiling sibling depending on the tiling direction.
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
        // Return once a suitable focus target is found.
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

/// Gets a focus target outside of the current workspace in the given
/// direction.
///
/// This will descend into the workspace in the given direction, and will
/// always return a tiling container. This makes it different from the
/// `focus_workspace` command with `FocusWorkspaceTarget::Direction`.
fn workspace_focus_target(
  origin_container: Container,
  direction: &Direction,
  state: &WmState,
) -> anyhow::Result<Option<Container>> {
  let monitor = origin_container.monitor().context("No monitor.")?;

  let target_workspace = state
    .monitor_in_direction(&monitor, direction)?
    .and_then(|monitor| monitor.displayed_workspace());

  let focus_target = target_workspace
    .as_ref()
    .and_then(|workspace| {
      workspace
        .descendant_in_direction(&direction.inverse())
        .map(Into::into)
    })
    .or(target_workspace.map(Into::into));

  Ok(focus_target)
}
