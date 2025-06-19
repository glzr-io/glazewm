use anyhow::Context;
use tracing::info;

use crate::{
  commands::monitor::add_monitor::move_workspace_to_monitor,
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Rebinds workspaces to their correct monitors based on the current monitor order
/// and workspace configuration. This ensures that workspaces bound to specific
/// monitor indices are on the correct monitors after monitor sorting or config changes.
pub fn rebind_workspaces_to_monitors(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let monitors = state.monitors();

  // Get all workspaces that should be bound to specific monitors
  let bound_workspace_configs = config
    .value
    .workspaces
    .iter()
    .filter(|config| config.bind_to_monitor.is_some())
    .collect::<Vec<_>>();

  for workspace_config in bound_workspace_configs {
    let monitor_index = workspace_config.bind_to_monitor.unwrap() as usize;

    // Find the workspace by name
    if let Some(workspace) = state.workspace_by_name(&workspace_config.name) {
      // Find the target monitor by its current index
      if let Some(target_monitor) = monitors.get(monitor_index) {
        let current_monitor = workspace.monitor().context("No monitor.")?;

        // Only move if the workspace is not already on the correct monitor
        if current_monitor.id() != target_monitor.id() {
          info!(
            "Moving workspace '{}' from monitor {} to monitor {} (index {})",
            workspace_config.name,
            current_monitor.native().device_name().unwrap_or("unknown"),
            target_monitor.native().device_name().unwrap_or("unknown"),
            monitor_index
          );

          move_workspace_to_monitor(
            &workspace,
            target_monitor,
            state,
            config,
          )?;
        }
      }
    }
  }

  Ok(())
}
