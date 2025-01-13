use anyhow::Context;
use tracing::info;
use wm_common::WmEvent;

use crate::{
  commands::{
    container::{detach_container, move_container_within_tree},
    workspace::sort_workspaces,
  },
  models::Monitor,
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

#[allow(clippy::needless_pass_by_value)]
pub fn remove_monitor(
  monitor: Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // TODO: Add monitor display trait.
  info!(
    "Removing monitor: {:?} {} {:?} {:?}",
    monitor.native().handle,
    monitor.native().device_name()?,
    monitor.native().device_path()?,
    monitor.native().hardware_id()?
  );

  let target_monitor = state
    .monitors()
    .into_iter()
    .find(|m| m.id() != monitor.id())
    .context("No target monitor to move workspaces.")?;

  // Avoid moving empty workspaces.
  let workspaces_to_move =
    monitor.workspaces().into_iter().filter(|workspace| {
      workspace.has_children() || workspace.config().keep_alive
    });

  for workspace in workspaces_to_move {
    // Move workspace to target monitor.
    move_container_within_tree(
      workspace.clone().into(),
      &target_monitor.clone().into(),
      target_monitor.child_count(),
      state,
    )?;

    sort_workspaces(&target_monitor, config)?;

    state.emit_event(WmEvent::WorkspaceUpdated {
      updated_workspace: workspace.to_dto()?,
    });
  }

  detach_container(monitor.clone().into())?;

  state.emit_event(WmEvent::MonitorRemoved {
    removed_id: monitor.id(),
    removed_device_name: monitor.native().device_name()?.to_string(),
  });

  Ok(())
}
