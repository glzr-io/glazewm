use anyhow::Context;
use tracing::info;

use crate::{
  common::Direction,
  containers::{commands::set_focused_descendant, traits::CommonGetters},
  monitors::Monitor,
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
    .and_then(|focused| focused.workspace())
    .context("No workspace is currently focused.")?;

  let target_workspace =
    target_workspace(target, &focused_workspace, state, config)?;

  if let Some(target_workspace) = target_workspace {
    info!("Focusing workspace: '{}'.", target_workspace.config().name);

    // Get the currently displayed workspace on the same monitor that the
    // workspace to focus is on.
    let displayed_workspace = target_workspace
      .monitor()
      .and_then(|monitor| monitor.displayed_workspace())
      .context("No workspace is currently displayed.")?;

    // Set focus to whichever window last had focus in workspace. If the
    // workspace has no windows, then set focus to the workspace itself.
    let container_to_focus = target_workspace
      .descendant_focus_order()
      .next()
      .unwrap_or_else(|| target_workspace.clone().into());

    set_focused_descendant(container_to_focus, None);
    state.pending_sync.focus_change = true;

    // Display the workspace to switch focus to.
    state
      .pending_sync
      .containers_to_redraw
      .extend([displayed_workspace.into(), target_workspace.into()]);

    // Get empty workspace to destroy (if one is found). Cannot destroy
    // empty workspaces if they're the only workspace on the monitor.
    let workspace_to_destroy =
      state.workspaces().into_iter().find(|workspace| {
        !workspace.config().keep_alive
          && !workspace.has_children()
          && !workspace.is_displayed()
      });

    if let Some(workspace) = workspace_to_destroy {
      deactivate_workspace(workspace, state)?;
    }

    // Save the currently focused workspace as recent.
    state.recent_workspace_name = Some(focused_workspace.config().name);
  }

  Ok(())
}

/// Gets the workspace to focus based on the given target.
///
/// If the target workspace is currently inactive, it gets activated on the
/// currently focused monitor.
fn target_workspace(
  target: FocusWorkspaceTarget,
  focused_workspace: &Workspace,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<Option<Workspace>> {
  let focused_monitor =
    focused_workspace.monitor().context("No focused monitor.")?;

  let target_workspace = match target {
    FocusWorkspaceTarget::Name(name) => {
      match focused_workspace.config().name == name {
        false => {
          name_to_workspace(&name, &focused_monitor, state, config)?
        }
        true if config.value.general.toggle_workspace_on_refocus => {
          recent_workspace(&focused_monitor, state, config)?
        }
        true => None,
      }
    }
    FocusWorkspaceTarget::Recent => {
      recent_workspace(&focused_monitor, state, config)?
    }
    FocusWorkspaceTarget::Next => {
      let workspaces = sorted_workspaces(state, config);
      let focused_index = workspace_index(&workspaces, focused_workspace)?;

      workspaces
        .get(focused_index + 1)
        .or_else(|| workspaces.first())
        .cloned()
    }
    FocusWorkspaceTarget::Previous => {
      let workspaces = sorted_workspaces(state, config);
      let focused_index = workspace_index(&workspaces, focused_workspace)?;

      workspaces
        .get(focused_index.checked_sub(1).unwrap_or(workspaces.len() - 1))
        .cloned()
    }
    FocusWorkspaceTarget::Direction(direction) => {
      let monitor =
        state.monitor_in_direction(&focused_monitor, &direction)?;

      monitor.and_then(|monitor| monitor.displayed_workspace())
    }
  };

  Ok(target_workspace)
}

/// Retrieves or activates a workspace by its name.
fn name_to_workspace(
  workspace_name: &str,
  target_monitor: &Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<Option<Workspace>> {
  match state.workspace_by_name(&workspace_name) {
    Some(workspace) => Ok(Some(workspace)),
    None => {
      activate_workspace(
        Some(&workspace_name),
        &target_monitor,
        state,
        config,
      )?;

      Ok(state.workspace_by_name(&workspace_name))
    }
  }
}

/// Gets the recent workspace based on `recent_workspace_name` in state.
fn recent_workspace(
  target_monitor: &Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<Option<Workspace>> {
  if let Some(recent_workspace_name) = state.recent_workspace_name.clone()
  {
    name_to_workspace(
      &recent_workspace_name,
      target_monitor,
      state,
      config,
    )
  } else {
    Ok(None)
  }
}

// Gets workspaces sorted by their position in the user config.
fn sorted_workspaces(
  state: &WmState,
  config: &UserConfig,
) -> Vec<Workspace> {
  let workspace_configs = &config.value.workspaces;
  let mut workspaces = state.workspaces();

  workspaces.sort_by_key(|workspace| {
    workspace_configs
      .iter()
      .position(|config| config.name == workspace.config().name)
  });

  workspaces
}

// Gets index of the given workspace within the vector of workspaces.
fn workspace_index(
  workspaces: &[Workspace],
  workspace: &Workspace,
) -> anyhow::Result<usize> {
  workspaces
    .iter()
    .position(|w| w.id() == workspace.id())
    .context("Failed to get index of given workspace.")
}
