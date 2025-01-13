use anyhow::Context;

use crate::{
  commands::workspace::focus_workspace, models::WorkspaceTarget,
  user_config::UserConfig, wm_state::WmState,
};

/// Focuses a monitor by a given monitor index.
pub fn focus_monitor(
  monitor_index: usize,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let monitors = state.monitors();

  let target_monitor = monitors.get(monitor_index).with_context(|| {
    format!("Monitor at index {monitor_index} was not found.")
  })?;

  let workspace_name = target_monitor
    .displayed_workspace()
    .map(|workspace| workspace.config().name)
    .context("Failed to get target workspace name.")?;

  focus_workspace(WorkspaceTarget::Name(workspace_name), state, config)
}
