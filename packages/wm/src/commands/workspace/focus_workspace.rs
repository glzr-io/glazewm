use anyhow::Context;
use tracing::info;

use super::activate_workspace;
use crate::{
  commands::{
    container::set_focused_descendant, workspace::deactivate_workspace,
  },
  models::WorkspaceTarget,
  traits::{CommonGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn focus_workspace(
  target: WorkspaceTarget,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_workspace = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .context("No workspace is currently focused.")?;

  // NEW LOGIC: Alternating / modulo-based filtering
  let manual_target_name = match target {
    WorkspaceTarget::NextOnMonitor | WorkspaceTarget::PreviousOnMonitor => {
      let current_monitor_id = focused_workspace.monitor().map(|m| m.id());
      let current_ws_name = focused_workspace.config().name.clone();

      if let Some(monitor_id) = current_monitor_id {
        // 1. Determine which monitor index we are on (0, 1, 2...)
        let monitors = state.monitors();
        let monitor_idx = monitors.iter().position(|m| m.id() == monitor_id).unwrap_or(0);
        let monitor_count = monitors.len();

        // 2. Filter config: Only keep workspaces that belong to this monitor index based on order
        // This creates the "1, 3, 5, 7" vs "2, 4, 6, 8" split automatically for 2 monitors.
        let valid_workspaces: Vec<String> = config.value.workspaces.iter()
            .enumerate()
            .filter(|(i, ws_config)| {
                // Priority 1: Strict Binding (if user set it)
                if let Some(bound_mon_idx) = ws_config.bind_to_monitor {
                     // Cast u32 to usize for comparison
                     return bound_mon_idx as usize == monitor_idx;
                }
                
                // Priority 2: Active on this monitor?
                if let Some(active_ws) = state.workspace_by_name(&ws_config.name) {
                    if let Some(active_mon) = active_ws.monitor() {
                        if active_mon.id() == monitor_id {
                            return true;
                        } else {
                            return false; // Active on ANOTHER monitor, strict exclude
                        }
                    }
                }

                // Priority 3: Alternating / Modulo Heuristic
                // If I am Monitor 0, keep indices 0, 2, 4...
                // If I am Monitor 1, keep indices 1, 3, 5...
                // Only applies if monitor_count > 0 to avoid div/0
                if monitor_count > 0 {
                    return i % monitor_count == monitor_idx;
                }

                true
            })
            .map(|(_, c)| c.name.clone())
            .collect();

        if !valid_workspaces.is_empty() {
            let current_idx = valid_workspaces.iter()
                .position(|name| *name == current_ws_name)
                .unwrap_or(0);

            let target_idx = match target {
                WorkspaceTarget::NextOnMonitor => (current_idx + 1) % valid_workspaces.len(),
                WorkspaceTarget::PreviousOnMonitor => {
                    if current_idx == 0 { valid_workspaces.len() - 1 } else { current_idx - 1 }
                }
                _ => 0,
            };

            Some(valid_workspaces[target_idx].clone())
        } else {
            None
        }
      } else {
        None
      }
    }
    _ => None,
  };

  let (target_workspace_name, target_workspace) =
    if let Some(name) = manual_target_name {
      // Check if the workspace is already active in the state
      if let Some(existing_ws) = state.workspace_by_name(&name) {
          (None, Some(existing_ws))
      } else {
          (Some(name), None) 
      }
    } else {
      state.workspace_by_target(&focused_workspace, target, config)?
    };

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

    let displayed_workspace = target_workspace
      .monitor()
      .and_then(|monitor| monitor.displayed_workspace())
      .context("No workspace is currently displayed.")?;

    let container_to_focus = target_workspace
      .descendant_focus_order()
      .next()
      .unwrap_or_else(|| target_workspace.clone().into());

    set_focused_descendant(&container_to_focus, None);
    state.pending_sync.queue_focus_change();

    state
      .pending_sync
      .queue_container_to_redraw(displayed_workspace)
      .queue_container_to_redraw(target_workspace);

    let workspace_to_destroy =
      state.workspaces().into_iter().find(|workspace| {
        !workspace.config().keep_alive
          && !workspace.has_children()
          && !workspace.is_displayed()
      });

    if let Some(workspace) = workspace_to_destroy {
      deactivate_workspace(workspace, state)?;
    }

    state.recent_workspace_name = Some(focused_workspace.config().name);
    state.pending_sync.queue_cursor_jump();
  }

  Ok(())
}