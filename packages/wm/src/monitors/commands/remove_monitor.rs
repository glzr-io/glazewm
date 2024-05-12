use anyhow::Context;
use tracing::info;

use crate::{
  containers::{
    commands::{detach_container, move_container_within_tree},
    traits::CommonGetters,
  },
  monitors::Monitor,
  wm_event::WmEvent,
  wm_state::WmState,
};

pub fn remove_monitor(
  monitor: Monitor,
  state: &mut WmState,
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
  let workspaces_to_move = monitor
    .children()
    .into_iter()
    .filter_map(|container| container.as_workspace().cloned())
    .filter(|workspace| {
      workspace.has_children() || workspace.config().keep_alive
    });

  for workspace in workspaces_to_move {
    // Move workspace to target monitor.
    move_container_within_tree(
      workspace.clone().into(),
      target_monitor.clone().into(),
      target_monitor.child_count(),
    )?;

    state.emit_event(WmEvent::WorkspaceMoved {
      workspace: workspace.clone(),
      new_monitor: target_monitor.clone(),
    });
  }

  detach_container(monitor.clone().into())?;

  state.emit_event(WmEvent::MonitorRemoved {
    removed_id: monitor.id(),
    removed_device_name: monitor.name()?.to_string(),
  });

  Ok(())
}
