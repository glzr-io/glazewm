use anyhow::Context;

use crate::{
  common::platform::NativeMonitor, containers::commands::attach_container,
  monitors::Monitor, user_config::UserConfig, wm_event::WmEvent,
  wm_state::WmState, workspaces::commands::activate_workspace,
};

pub fn add_monitor(
  native_monitor: NativeMonitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // Create `Monitor` instance. This uses the working area of the monitor
  // instead of the bounds of the display. The working area excludes
  // taskbars and other reserved display space.
  let monitor = Monitor::new(native_monitor);

  attach_container(
    &monitor.clone().into(),
    &state.root_container.clone().into(),
    None,
  )?;

  state.emit_event(WmEvent::MonitorAdded {
    added_monitor: monitor.clone(),
  });

  // Activate a workspace on the newly added monitor.
  activate_workspace(None, &monitor, state, config)
    .context("At least 1 workspace is required per monitor.")?;

  Ok(())
}
