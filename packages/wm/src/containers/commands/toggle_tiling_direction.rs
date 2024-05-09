use anyhow::Context;
use uuid::Uuid;

use crate::{
  common::TilingDirection,
  containers::{
    traits::{CommonGetters, TilingDirectionGetters, TilingSizeGetters},
    Container, DirectionContainer, SplitContainer,
  },
  user_config::UserConfig,
  windows::TilingWindow,
  wm_event::WmEvent,
  wm_state::WmState,
  workspaces::Workspace,
};

use super::{
  attach_container, detach_container, flatten_split_container,
  replace_container,
};

pub fn toggle_tiling_direction(
  container: Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let (modified_id, new_tiling_direction) = match container {
    Container::TilingWindow(tiling_window) => {
      toggle_window_direction(tiling_window, state, config)
    }
    Container::Workspace(workspace) => {
      toggle_workspace_direction(workspace, state)
    }
    // Can only toggle tiling direction from a tiling window or workspace.
    _ => return Ok(()),
  }?;

  state.emit_event(WmEvent::TilingDirectionChanged {
    modified_id,
    new_tiling_direction,
  });

  Ok(())
}

fn toggle_window_direction(
  tiling_window: TilingWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<(Uuid, TilingDirection)> {
  let parent = tiling_window
    .direction_container()
    .context("No direction container.")?;

  // If the window is an only child, then either change the tiling
  // direction of its parent workspace or flatten its parent split
  // container.
  if !tiling_window.has_siblings() {
    match parent {
      DirectionContainer::Workspace(workspace) => {
        return toggle_workspace_direction(workspace, state);
      }
      DirectionContainer::Split(split_container) => {
        flatten_split_container(split_container.clone())?;

        return Ok((
          split_container.id(),
          split_container.tiling_direction().inverse(),
        ));
      }
    };
  }

  // Create a new split container to wrap the window.
  let split_container = SplitContainer::new(
    parent.tiling_direction().inverse(),
    config.value.gaps.inner_gap.clone(),
  );

  // Replace the window with the wrapping split container. The window
  // has to be attached to the split container after the replacement.
  replace_container(
    split_container.clone().into(),
    parent.clone().into(),
    tiling_window.index(),
  )?;

  // Add the window as a child to the new split container.
  attach_container(
    &tiling_window.clone().into(),
    &split_container.clone().into(),
    None,
  )?;

  Ok((split_container.id(), split_container.tiling_direction()))
}

fn toggle_workspace_direction(
  workspace: Workspace,
  state: &mut WmState,
) -> anyhow::Result<(Uuid, TilingDirection)> {
  workspace.set_tiling_direction(workspace.tiling_direction().inverse());

  let inverse_split_containers = workspace
    .children()
    .iter()
    .filter_map(|child| match child.as_split() {
      Some(split)
        if split.tiling_direction() == workspace.tiling_direction() =>
      {
        Some(split)
      }
      _ => None,
    })
    .cloned()
    .collect::<Vec<_>>();

  // Flatten any top-level split containers with the same tiling
  // direction as the workspace.
  for split_container in inverse_split_containers {
    flatten_split_container(split_container)?;
  }

  state.add_container_to_redraw(workspace.clone().into());

  Ok((workspace.id(), workspace.tiling_direction()))
}
