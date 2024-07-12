use anyhow::Context;
use uuid::Uuid;

use super::{flatten_split_container, wrap_in_split_container};
use crate::{
  common::TilingDirection,
  containers::{
    traits::{CommonGetters, TilingDirectionGetters},
    Container, DirectionContainer, SplitContainer,
  },
  user_config::UserConfig,
  windows::TilingWindow,
  wm_event::WmEvent,
  wm_state::WmState,
  workspaces::Workspace,
};

pub fn toggle_tiling_direction(
  container: Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let (modified_id, new_tiling_direction) = match container {
    Container::TilingWindow(tiling_window) => {
      toggle_window_direction(tiling_window, config)
    }
    Container::Workspace(workspace) => {
      toggle_workspace_direction(workspace)
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
  config: &UserConfig,
) -> anyhow::Result<(Uuid, TilingDirection)> {
  let parent = tiling_window
    .direction_container()
    .context("No direction container.")?;

  // If the window is an only child, then either change the tiling
  // direction of its parent workspace or flatten its parent split
  // container.
  if tiling_window.tiling_siblings().count() == 0 {
    match parent {
      DirectionContainer::Workspace(workspace) => {
        return toggle_workspace_direction(workspace);
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

  wrap_in_split_container(
    split_container.clone(),
    parent.into(),
    vec![tiling_window.clone().into()],
  )?;

  Ok((split_container.id(), split_container.tiling_direction()))
}

fn toggle_workspace_direction(
  workspace: Workspace,
) -> anyhow::Result<(Uuid, TilingDirection)> {
  workspace.set_tiling_direction(workspace.tiling_direction().inverse());

  Ok((workspace.id(), workspace.tiling_direction()))
}
