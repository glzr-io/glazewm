use anyhow::Context;
use tracing::info;
use wm_common::WmEvent;

use crate::{
  commands::{
    container::move_container_within_tree,
    workspace::sort_workspaces,
  },
  models::{Monitor, WindowContainer},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_system_resume(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("System resumed from sleep, restoring monitor assignments");

  // Skip if no saved assignments
  if state.sleep_monitor_assignments.is_empty() {
    return Ok(());
  }

  let windows = state.windows();

  for window in windows {
    // Skip windows without saved assignments
    let window_id = window.id();
    let Some(saved_hardware_id) = state.get_saved_monitor_assignment(&window_id) else {
      continue;
    };

    // Find the current monitor for this window
    let current_workspace = match window.workspace() {
      Some(workspace) => workspace,
      None => continue,
    };

    let current_monitor = match current_workspace.monitor() {
      Some(monitor) => monitor,
      None => continue,
    };

    // Get the current hardware ID
    let current_hardware_id = match current_monitor.native().hardware_id() {
      Ok(Some(id)) => id.clone(),
      _ => continue,
    };

    // Skip if already on the correct monitor
    if current_hardware_id == *saved_hardware_id {
      continue;
    }

    // Find the target monitor with the saved hardware ID
    let target_monitor = state.monitors().into_iter().find(|monitor| {
      if let Ok(Some(id)) = monitor.native().hardware_id() {
        id == *saved_hardware_id
      } else {
        false
      }
    });

    // Move the window to the target monitor's active workspace
    if let Some(target_monitor) = target_monitor {
      if let Some(target_workspace) = target_monitor.displayed_workspace() {
        // Move the window to the target workspace
        move_container_to_workspace(&window, &target_workspace, state, config)?;

        state.emit_event(WmEvent::WindowMoved {
          window_id: window.id(),
          workspace_id: target_workspace.id(),
        });
      }
    }
  }

  // Clear saved assignments after restoration
  state.sleep_monitor_assignments.clear();

  Ok(())
}

fn move_container_to_workspace(
  window: &WindowContainer,
  target_workspace: &crate::models::Workspace,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // Move window to target workspace
  move_container_within_tree(
    &window.clone().into(),
    &target_workspace.clone().into(),
    target_workspace.child_count(),
    state,
  )?;

  // Sort workspaces after moving
  if let Some(monitor) = target_workspace.monitor() {
    sort_workspaces(&monitor, config)?;
  }

  // Queue redraw
  state.pending_sync.queue_container_to_redraw(window.clone().into());

  Ok(())
}
