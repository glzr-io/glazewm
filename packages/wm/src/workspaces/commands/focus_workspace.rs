use anyhow::Context;
use tracing::info;

use crate::{
  containers::{commands::set_focused_descendant, traits::CommonGetters},
  user_config::UserConfig,
  wm_state::WmState,
  workspaces::{
    commands::{activate_workspace, deactivate_workspace},
    WorkspaceTarget,
  },
};

/// Focuses a workspace by a given target.
///
/// This target can be a workspace name, the most recently focused
/// workspace, the next workspace, the previous workspace, or the workspace
/// in a given direction from the currently focused workspace.
///
/// The workspace will be activated if it isn't already active.
pub fn focus_workspace(
  target: WorkspaceTarget,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_workspace = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .context("No workspace is currently focused.")?;

  let (target_workspace_name, target_workspace) =
    state.workspace_by_target(&focused_workspace, target, config)?;

  // Retrieve or activate the target workspace by its name.
  let target_workspace = match target_workspace {
    Some(_) => anyhow::Ok(target_workspace),
    _ => match target_workspace_name {
      Some(name) => {
        activate_workspace(Some(&name), None, state, config)?;

        Ok(state.workspace_by_name(&name))
      }
      _ => Ok(None),
    },
  }?;

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

    set_focused_descendant(container_to_focus.clone(), None);
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
    state.pending_sync.cursor_jump = true;
  }

  Ok(())
}
