use anyhow::Context;

use crate::{
  user_config::UserConfig,
  wm_state::WmState,
  workspaces::{commands::focus_workspace, WorkspaceTarget},
};

/// Focuses a monitor by a given monitor index.
pub fn focus_monitor(
  monitor_index: usize,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if state.paused {
    return Ok(());
  }

  let monitors = state.monitors();

  let target_monitor = monitors.get(monitor_index).with_context(|| {
    format!("Monitor at index {} was not found.", monitor_index)
  })?;

  let workspace_name = target_monitor
    .displayed_workspace()
    .map(|workspace| workspace.config().name)
    .context("Failed to get target workspace name.")?;

  focus_workspace(WorkspaceTarget::Name(workspace_name), state, config)
}
