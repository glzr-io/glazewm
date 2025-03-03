use anyhow::Context;

use super::{focus_workspace, move_workspace_to_monitor};
use crate::{
  models::{MonitorTarget, Workspace, WorkspaceTarget},
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Swap workspace on origin and target
pub fn swap_workspace(
  origin_workspace: &Workspace,
  target: MonitorTarget,
  change_focus: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let origin_monitor = origin_workspace
    .monitor()
    .context("Workspace have no monitor.")?;

  let target_monitor = match target {
    MonitorTarget::Direction(direction) => {
      state.monitor_in_direction(&origin_monitor, &direction)?
    }
    MonitorTarget::Index(index) => {
      let monitors = state.monitors();
      monitors.get(index).cloned()
    }
    MonitorTarget::Monitor(monitor) => Some(monitor),
  }
  .context("There is no valid monitor")?;

  let target_workspace = target_monitor
    .displayed_workspace()
    .context("Target monitor have no displayed workspace.")?;

  move_workspace_to_monitor(
    &target_workspace,
    MonitorTarget::Monitor(origin_monitor),
    state,
    config,
  )?;

  move_workspace_to_monitor(
    origin_workspace,
    MonitorTarget::Monitor(target_monitor),
    state,
    config,
  )?;

  if change_focus {
    focus_workspace(
      WorkspaceTarget::Name(target_workspace.config().name),
      false,
      state,
      config,
    )?;
  }

  Ok(())
}
