use anyhow::Context;
use tracing::info;
use wm_common::WindowState;

use crate::{
  commands::{
    container::{move_container_within_tree, set_focused_descendant},
    window::manage_window::rebuild_spiral_layout,
    workspace::activate_workspace,
  },
  models::{TilingWindow, WindowContainer, WorkspaceTarget},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn move_window_to_workspace(
  window: WindowContainer,
  target: WorkspaceTarget,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let current_workspace = window.workspace().context("No workspace.")?;
  let current_monitor =
    current_workspace.monitor().context("No monitor.")?;

  // NEW LOGIC: Alternating / modulo-based filtering
  let manual_target_name = match target {
    WorkspaceTarget::NextOnMonitor | WorkspaceTarget::PreviousOnMonitor => {
      let monitor_id = current_monitor.id();
      let current_ws_name = current_workspace.config().name.clone();

      // 1. Determine which monitor index we are on
      let monitors = state.monitors();
      let monitor_idx = monitors.iter().position(|m| m.id() == monitor_id).unwrap_or(0);
      let monitor_count = monitors.len();

      // 2. Filter config: Only keep workspaces for this monitor
      let valid_workspaces: Vec<String> = config.value.workspaces.iter()
          .enumerate()
          .filter(|(i, ws_config)| {
                // Priority 1: Strict Binding
                if let Some(bound_mon_idx) = ws_config.bind_to_monitor {
                     return bound_mon_idx as usize == monitor_idx;
                }
                
                // Priority 2: Active on this monitor?
                if let Some(active_ws) = state.workspace_by_name(&ws_config.name) {
                    if let Some(active_mon) = active_ws.monitor() {
                        if active_mon.id() == monitor_id {
                            return true;
                        } else {
                            return false; 
                        }
                    }
                }

                // Priority 3: Alternating / Modulo Heuristic
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
    }
    _ => None,
  };

  let (target_workspace_name, target_workspace) =
    if let Some(name) = manual_target_name {
      // Check if the workspace is already active
      if let Some(existing_ws) = state.workspace_by_name(&name) {
          (None, Some(existing_ws))
      } else {
          (Some(name), None) 
      }
    } else {
      state.workspace_by_target(&current_workspace, target, config)?
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
    if target_workspace.id() == current_workspace.id() {
      return Ok(());
    }

    info!(
      "Moving window to workspace: '{}'.",
      target_workspace.config().name
    );

    let target_monitor =
      target_workspace.monitor().context("No monitor.")?;

    if current_monitor
      .has_dpi_difference(&target_monitor.clone().into())?
    {
      window.set_has_pending_dpi_adjustment(true);
    }

    if target_monitor.id() != current_monitor.id() {
      window.set_floating_placement(
        window
          .floating_placement()
          .translate_to_center(&target_workspace.to_rect()?),
      );
    }

    if let WindowContainer::NonTilingWindow(window) = &window {
      window.set_insertion_target(None);
    }

    let focus_target = state.focus_target_after_removal(&window);

    let focus_reset_target = if target_workspace.is_displayed() {
      None
    } else {
      target_monitor.descendant_focus_order().next()
    };

    let insertion_sibling = target_workspace
      .descendant_focus_order()
      .filter_map(|descendant| descendant.as_window_container().ok())
      .find(|descendant| descendant.state() == WindowState::Tiling);

    match (window.is_tiling_window(), insertion_sibling.is_some()) {
      (true, true) => {
        if let Some(insertion_sibling) = insertion_sibling {
          match insertion_sibling.parent() {
            Some(parent) => {
              move_container_within_tree(
                &window.clone().into(),
                &parent,
                insertion_sibling.index() + 1,
                state,
              )?;
            }
            None => {
              move_container_within_tree(
                &window.clone().into(),
                &target_workspace.clone().into(),
                target_workspace.child_count(),
                state,
              )?;
            }
          }
        }
      }
      _ => {
        move_container_within_tree(
          &window.clone().into(),
          &target_workspace.clone().into(),
          target_workspace.child_count(),
          state,
        )?;
      }
    }

    if let Some(focus_reset_target) = focus_reset_target {
      set_focused_descendant(
        &focus_reset_target,
        Some(&target_monitor.into()),
      );
    }

    if let Some(focus_target) = focus_target {
      set_focused_descendant(&focus_target, None);
      state.pending_sync.queue_focus_change();
    }

    match window {
      WindowContainer::NonTilingWindow(_) => {
        state.pending_sync.queue_container_to_redraw(window);
      }
      WindowContainer::TilingWindow(_) => {
        let source_windows: Vec<TilingWindow> = current_workspace
          .descendants()
          .filter_map(|c| c.try_into().ok())
          .collect();
        if !source_windows.is_empty() {
          rebuild_spiral_layout(&current_workspace, &source_windows)?;
        }

        let target_windows: Vec<TilingWindow> = target_workspace
          .descendants()
          .filter_map(|c| c.try_into().ok())
          .collect();
        if !target_windows.is_empty() {
          rebuild_spiral_layout(&target_workspace, &target_windows)?;
        }

        state
          .pending_sync
          .queue_containers_to_redraw(current_workspace.tiling_children())
          .queue_containers_to_redraw(target_workspace.tiling_children());
      }
    }

    state
      .pending_sync
      .queue_workspace_to_reorder(target_workspace);
  }

  Ok(())
}