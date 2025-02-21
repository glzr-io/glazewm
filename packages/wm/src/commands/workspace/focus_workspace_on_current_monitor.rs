use anyhow::Context;
use tracing::info;
use wm_common::WmEvent;

use super::activate_workspace;
use crate::{
  commands::{
    container::{move_container_within_tree, set_focused_descendant},
    workspace::{deactivate_workspace, sort_workspaces},
  },
  models::WorkspaceTarget,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Focuses a workspace by a given target.
///
/// If the target workspace and focused workspace is in the same monitor,
/// does the same thing as `focus_workspace` function.
///
/// If the target workspace is on a different monitor,
/// it will move target workspace to the focused monitor and focuses it.
///
/// If the target workspace is displayed on a different monitor,
/// it will swap the target workspace and the focused workspace and focuse
/// the target workspace.
pub fn focus_workspace_on_current_monitor(
  target: WorkspaceTarget,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_workspace = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .context("No workspace is currently focused.")?;

  let focused_monitor = focused_workspace
    .monitor()
    .context("Workspace has no parent monitor.")?;

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
    info!("Focusing workspace: {target_workspace}");
    let target_monitor = target_workspace
      .monitor()
      .context("Focused workspace has no perent monitor.")?;

    if focused_monitor.id() == target_monitor.id() {
      // Does the same thing as `focus_workspace`
      normal_focus(state, &target_workspace, &focused_workspace);
    } else if target_workspace.is_displayed() {
      swap_and_focus(state, config, &target_workspace, &focused_workspace)?;
    } else {
      move_and_focus(state, config, &target_workspace, &focused_workspace)?;
    }

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
  }
  Ok(())
}

fn normal_focus(
  state: &mut WmState,
  target_workspace: &crate::models::Workspace,
  focused_workspace: &crate::models::Workspace,
) {
  info!("Normal focus: {target_workspace}");

  let container_to_focus = target_workspace
    .descendant_focus_order()
    .next()
    .unwrap_or_else(|| target_workspace.clone().into());

  set_focused_descendant(&container_to_focus, None);


  state
    .pending_sync
    .queue_focus_change()
    .queue_container_to_redraw(focused_workspace.clone())
    .queue_container_to_redraw(target_workspace.clone())
    .queue_cursor_jump();

  state.recent_workspace_name = Some(target_workspace.config().name);
}

fn move_and_focus(
  state: &mut WmState,
  config: &UserConfig,
  target_workspace: &crate::models::Workspace,
  focused_workspace: &crate::models::Workspace,
) -> anyhow::Result<()> {
  info!("Move focus: {target_workspace}");
  let focused_monitor = focused_workspace
    .monitor()
    .context("Workspace has no monitor")?;

  move_container_within_tree(
    &target_workspace.clone().as_container(),
    &focused_monitor.clone().as_container(),
    focused_monitor.child_count(),
    state,
  )?;

  sort_workspaces(&focused_monitor, config)?;

  let windows = target_workspace
    .descendants()
    .filter_map(|descendant| descendant.as_window_container().ok());

  for window in windows {
    window.set_has_pending_dpi_adjustment(true);

    window.set_floating_placement(
      window
        .floating_placement()
        .translate_to_center(&target_workspace.to_rect()?),
    );
  }

  let container_to_focus = target_workspace
    .descendant_focus_order()
    .next()
    .unwrap_or_else(|| target_workspace.clone().into());

  set_focused_descendant(&container_to_focus, None);

  state
    .pending_sync
    .queue_focus_change()
    .queue_container_to_redraw(target_workspace.clone())
    .queue_container_to_redraw(focused_workspace.clone())
    .queue_cursor_jump();

  state.emit_event(WmEvent::WorkspaceUpdated {
    updated_workspace: target_workspace.to_dto()?
  });

  state.recent_workspace_name = Some(target_workspace.config().name);

  Ok(())
}

fn swap_and_focus(
  state: &mut WmState,
  config: &UserConfig,
  target_workspace: &crate::models::Workspace,
  focused_workspace: &crate::models::Workspace,
) -> anyhow::Result<()> {
  info!("Swap focus: swap {target_workspace} and {focused_workspace}");
  let focused_monitor = focused_workspace
    .monitor()
    .context("Workspace has no monitor")?;

  let target_monitor = target_workspace
    .monitor()
    .context("Workspace has no monitor")?;

  move_container_within_tree(
    &target_workspace.clone().as_container(),
    &focused_monitor.clone().as_container(),
    focused_monitor.child_count(),
    state,
  )?;

  move_container_within_tree(
    &focused_workspace.clone().as_container(),
    &target_monitor.clone().as_container(),
    target_monitor.child_count(),
    state
  )?;

  sort_workspaces(&focused_monitor, config)?;
  sort_workspaces(&target_monitor, config)?;


  let windows = target_workspace
    .descendants()
    .filter_map(|descendant| descendant.as_window_container().ok());

  for window in windows {
    window.set_has_pending_dpi_adjustment(true);

    window.set_floating_placement(
      window
        .floating_placement()
        .translate_to_center(&target_workspace.to_rect()?),
    );
  }

  let windows = focused_workspace
    .descendants()
    .filter_map(|descendant| descendant.as_window_container().ok());

  for window in windows {
    window.set_has_pending_dpi_adjustment(true);

    window.set_floating_placement(
      window
        .floating_placement()
        .translate_to_center(&focused_workspace.to_rect()?),
    );
  }

  let container_to_focus = target_workspace
    .descendant_focus_order()
    .next()
    .unwrap_or_else(|| target_workspace.clone().into());

  set_focused_descendant(&container_to_focus, None);

  state
    .pending_sync
    .queue_focus_change()
    .queue_container_to_redraw(target_workspace.clone())
    .queue_container_to_redraw(focused_workspace.clone())
    .queue_cursor_jump();

  state.emit_event(WmEvent::WorkspaceUpdated {
    updated_workspace: focused_workspace.to_dto()?
  });

  state.emit_event(WmEvent::WorkspaceUpdated {
    updated_workspace: target_workspace.to_dto()?
  });

  state.recent_workspace_name = Some(target_workspace.config().name);
  Ok(())
}
