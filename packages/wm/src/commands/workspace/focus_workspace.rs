use anyhow::Context;
use tracing::info;

use super::{activate_workspace, swap_workspace_internal};
use crate::{
  commands::{
    container::set_focused_descendant,
    workspace::{deactivate_workspace, move_workspace_to_monitor},
  },
  models::{MonitorTarget, Workspace, WorkspaceTarget},
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
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
  summon_to_current_monitor: bool,
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
    if summon_to_current_monitor {
      let target_monitor = target_workspace
        .monitor()
        .context("Focused workspace has no perent monitor.")?;

      let focused_monitor = focused_workspace
        .monitor()
        .context("Workspace has no parent monitor.")?;

      if focused_monitor.id() == target_monitor.id() {
        normal_focus(state, &target_workspace, &focused_workspace)?;
      } else if target_workspace.is_displayed() {
        swap_and_focus(
          state,
          config,
          &target_workspace,
          &focused_workspace,
        )?;
      } else {
        move_and_focus(
          state,
          config,
          &target_workspace,
          &focused_workspace,
        )?;
      }
    } else {
      normal_focus(state, &target_workspace, &focused_workspace)?;
    }
  }

  Ok(())
}

fn normal_focus(
  state: &mut WmState,
  target_workspace: &crate::models::Workspace,
  focused_workspace: &crate::models::Workspace,
) -> anyhow::Result<()> {
  info!("Normal focus: {target_workspace}");

  let container_to_focus = target_workspace
    .descendant_focus_order()
    .next()
    .unwrap_or_else(|| target_workspace.clone().into());

  set_focused_descendant(&container_to_focus, None);

  state.recent_workspace_name = Some(target_workspace.config().name);
  state
    .pending_sync
    .queue_focus_change()
    .queue_container_to_redraw(focused_workspace.clone())
    .queue_container_to_redraw(target_workspace.clone())
    .queue_cursor_jump();

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

  Ok(())
}

fn move_and_focus(
  state: &mut WmState,
  config: &UserConfig,
  target_workspace: &Workspace,
  focused_workspace: &Workspace,
) -> anyhow::Result<()> {
  info!("Move focus: {target_workspace}");
  let focused_monitor = focused_workspace
    .monitor()
    .context("Workspace has no monitor")?;

  move_workspace_to_monitor(
    target_workspace,
    MonitorTarget::Monitor(focused_monitor),
    state,
    config,
  )?;

  // Make sure that it is focused after move
  let container_to_focus = target_workspace
    .descendant_focus_order()
    .next()
    .unwrap_or_else(|| target_workspace.clone().as_container());

  set_focused_descendant(&container_to_focus, None);

  Ok(())
}

fn swap_and_focus(
  state: &mut WmState,
  config: &UserConfig,
  target_workspace: &Workspace,
  focused_workspace: &Workspace,
) -> anyhow::Result<()> {
  info!("Move focus: {target_workspace}");

  let target_monitor = target_workspace
    .monitor()
    .context("Workspace has no monitor")?;

  let focused_monitor = focused_workspace
    .monitor()
    .context("Workspace has no monitor")?;

  swap_workspace_internal(
    &target_monitor.into(),
    &focused_monitor.into(),
    true,
    state,
    config,
  )
}
