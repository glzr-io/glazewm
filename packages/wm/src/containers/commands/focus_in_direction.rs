use crate::{
  common::Direction,
  containers::{traits::CommonGetters, Container},
  user_config::UserConfig,
  windows::{traits::WindowGetters, TilingWindow, WindowState},
  wm_state::WmState,
  workspaces::{
    commands::{focus_workspace, FocusWorkspaceTarget},
    Workspace,
  },
};

use super::set_focused_descendant;

pub fn focus_in_direction(
  origin_container: Container,
  direction: Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focus_target = match origin_container {
    Container::TilingWindow(_) => {
      focus_target_from_tiling(origin_container, direction, state, config)
    }
    Container::NonTilingWindow(non_tiling_window) => {
      match non_tiling_window.state() {
        // WindowState::Floating(floating_window) => {
        //   focus_target_from_floating(origin_container, direction)
        // }
        // WindowState::Fullscreen(_) => {
        //   return focus_workspace(
        //     FocusWorkspaceTarget::Direction(direction),
        //     state,
        //     config,
        //   );
        // }
        _ => None,
      }
    }
    Container::Workspace(_) => {
      return focus_workspace(
        FocusWorkspaceTarget::Direction(direction),
        state,
        config,
      );
    }
    _ => None,
  };

  if let Some(focus_target) = focus_target {
    set_focused_descendant(focus_target, None);
    state.has_pending_focus_sync = true;
  }

  Ok(())
}

fn focus_target_from_floating(
  origin_container: Container,
  direction: Direction,
) -> Option<Container> {
  let floating_siblings = origin_container
    .siblings()
    .filter_map(|s| s.as_non_tiling_window().cloned())
    .filter(|w| matches!(w.state(), WindowState::Floating(_)))
    .collect::<Vec<_>>();

  match direction {
    Direction::Left => {
      // Wrap if next/previous floating window is not found.
      origin_container
        .next_siblings()
        .filter_map(|s| s.as_non_tiling_window().cloned())
        .filter(|w| matches!(w.state(), WindowState::Floating(_)))
        .next()
        .or_else(|| floating_siblings.last().cloned())
        .map(|c| c.into())
    }
    Direction::Right => {
      // Wrap if next/previous floating window is not found.
      origin_container
        .prev_siblings()
        .filter_map(|s| s.as_non_tiling_window().cloned())
        .filter(|w| matches!(w.state(), WindowState::Floating(_)))
        .next()
        .or_else(|| floating_siblings.first().cloned())
        .map(|c| c.into())
    }
    // Cannot focus vertically from a floating window.
    _ => None,
  }
}

fn focus_target_from_tiling(
  origin_container: Container,
  direction: Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> Option<Container> {
  let focus_target_within_workspace =
    focus_target_within_workspace(origin_container, &direction);

  if focus_target_within_workspace.is_some() {
    return focus_target_within_workspace;
  }

  // If a suitable focus target isn't found in the current workspace, attempt to find
  // a workspace in the given direction.
  focus_target_outside_workspace(direction)
}

/// Attempt to find a focus target within the focused workspace. Traverse upwards from the
/// focused container to find an adjacent container that can be focused.
fn focus_target_within_workspace(
  origin_container: Container,
  direction: &Direction,
) -> Option<Container> {
  todo!()
  //   let tiling_direction = direction.get_tiling_direction();
  //   let mut focus_reference = origin_container.clone();

  //   // Traverse upwards from the focused container. Stop searching when a workspace is
  //   // encountered.
  //   while !focus_reference.is_workspace() {
  //     let parent = focus_reference
  //       .parent()
  //       .unwrap()
  //       .as_split_container()
  //       .unwrap();

  //     if !focus_reference.has_siblings()
  //       || parent.tiling_direction() != tiling_direction
  //     {
  //       focus_reference = parent.clone().into();
  //       continue;
  //     }

  //     let focus_target =
  //       if direction == Direction::Up || direction == Direction::Left {
  //         focus_reference.previous_sibling_of_type::<dyn IResizable>()
  //       } else {
  //         focus_reference.next_sibling_of_type::<dyn IResizable>()
  //       };

  //     if focus_target.is_none() {
  //       focus_reference = parent.into();
  //       continue;
  //     }

  //     return container_service.get_descendant_in_direction(
  //       &focus_target.unwrap(),
  //       direction.inverse(),
  //     );
  //   }

  //   None
}

/// Attempt to find a focus target in a different workspace than the focused workspace.
fn focus_target_outside_workspace(
  direction: Direction,
) -> Option<Container> {
  todo!()
  // let focused_monitor = monitor_service.get_focused_monitor();

  // let monitor_in_direction =
  //   monitor_service.get_monitor_in_direction(direction, &focused_monitor);
  // let workspace_in_direction =
  //   monitor_in_direction.and_then(|m| m.displayed_workspace());

  // if workspace_in_direction.is_none() {
  //   return None;
  // }

  // container_service.get_descendant_in_direction(
  //   &workspace_in_direction.unwrap(),
  //   direction.inverse(),
  // )
}
