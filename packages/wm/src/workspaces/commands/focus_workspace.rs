use anyhow::Context;
use tracing::info;

use crate::{
  common::Direction,
  containers::{commands::set_focused_descendant, traits::CommonGetters},
  user_config::UserConfig,
  wm_state::WmState,
  workspaces::{
    commands::{activate_workspace, deactivate_workspace},
    Workspace,
  },
};

pub enum FocusWorkspaceTarget {
  Name(String),
  Recent,
  Next,
  Previous,
  Direction(Direction),
}

/// Focuses a workspace by a given target. This target can be a workspace
/// name, the most recently focused workspace, the next workspace, the
/// previous workspace, or the workspace in a given direction from the
/// currently focused workspace.
///
/// The workspace will be activated if it isn't already active.
pub fn focus_workspace(
  target: FocusWorkspaceTarget,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_workspace = state
    .focused_container()
    .and_then(|c| c.workspace())
    .context("No workspace is currently focused.")?;

  let workspace_name =
    target_workspace_name(target, &focused_workspace, state, config)?;

  if let Some(workspace_name) = workspace_name {
    // Get the workspace to focus. If it's currently inactive, then
    // activate it on the currently focused monitor.
    let workspace_to_focus =
      match state.workspace_by_name(&workspace_name) {
        Some(workspace) => Some(workspace),
        None => {
          let focused_monitor =
            focused_workspace.monitor().context("No focused monitor.")?;

          activate_workspace(
            Some(&workspace_name),
            &focused_monitor,
            state,
            config,
          )?;

          state.workspace_by_name(&workspace_name)
        }
      }
      .context("Failed to get workspace to focus.")?;

    let displayed_workspace = workspace_to_focus
      .monitor()
      .and_then(|monitor| monitor.displayed_workspace())
      .context("No workspace is currently displayed.")?;

    // Save the currently focused workspace as recent.
    state.recent_workspace_name = Some(focused_workspace.config().name);

    info!(
      "Focusing workspace: '{}'.",
      workspace_to_focus.config().name
    );

    // Set focus to the last focused window in workspace. If the workspace
    // has no descendant windows, then set focus to the workspace itself.
    let container_to_focus = workspace_to_focus
      .last_focused_descendant()
      .unwrap_or_else(|| workspace_to_focus.clone().into());

    set_focused_descendant(container_to_focus, None);
    state.has_pending_focus_sync = true;

    // Display the workspace to switch focus to.
    state.containers_to_redraw.push(displayed_workspace.into());
    state.containers_to_redraw.push(workspace_to_focus.into());

    // Get empty workspace to destroy (if one is found). Cannot destroy empty
    // workspaces if they're the only workspace on the monitor.
    let workspace_to_destroy =
      state.workspaces().into_iter().find(|workspace| {
        !workspace.config().keep_alive
          && !workspace.has_children()
          && !workspace.is_displayed()
      });

    if let Some(workspace) = workspace_to_destroy {
      deactivate_workspace(workspace, state)?;
    }
  }
  Ok(())
}

/// Gets the name of the workspace to focus.
fn target_workspace_name(
  target: FocusWorkspaceTarget,
  focused_workspace: &Workspace,
  state: &WmState,
  config: &UserConfig,
) -> anyhow::Result<Option<String>> {
  let workspace_configs = &config.value.workspaces;
  let mut workspaces = state.workspaces();

  // Sort workspaces by their position in the user config.
  workspaces.sort_by_key(|workspace| {
    workspace_configs
      .iter()
      .position(|config| config.name == workspace.config().name)
  });

  // Get index of the currently focused workspace within the sorted vector
  // of workspaces.
  let focused_index = workspaces
    .iter()
    .position(|workspace| workspace.id() == focused_workspace.id())
    .context("Unable to get config position of focused workspace.")?;

  let workspace_name = match target {
    FocusWorkspaceTarget::Name(name) => {
      match focused_workspace.config().name == name {
        false => Some(name),
        true if config.value.general.toggle_workspace_on_refocus => {
          state.recent_workspace_name.clone()
        }
        true => None,
      }
    }
    FocusWorkspaceTarget::Recent => state.recent_workspace_name.clone(),
    FocusWorkspaceTarget::Next => {
      let index = match focused_index == workspaces.len() - 1 {
        true => 0,
        _ => focused_index + 1,
      };

      workspaces.get(index).map(|w| w.config().name.clone())
    }
    FocusWorkspaceTarget::Previous => {
      let index = match focused_index {
        0 => workspaces.len() - 1,
        _ => focused_index - 1,
      };

      workspaces.get(index).map(|w| w.config().name.clone())
    }
    FocusWorkspaceTarget::Direction(direction) => {
      let focused_monitor =
        focused_workspace.monitor().context("No focused monitor.")?;

      let monitor =
        state.monitor_in_direction(direction, &focused_monitor)?;

      monitor
        .and_then(|m| m.displayed_workspace())
        .map(|w| w.config().name.clone())
    }
  };

  Ok(workspace_name)
}
