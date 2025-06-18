use anyhow::Context;
use tracing::{info, warn};
use wm_common::WmEvent;
use wm_platform::NativeMonitor;

use crate::{
  commands::{container::attach_container, workspace::activate_workspace},
  models::Monitor,
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
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

  info!("Monitor added: {monitor}");

  state.emit_event(WmEvent::MonitorAdded {
    added_monitor: monitor.to_dto()?,
  });

  let bound_workspace_configs = config
    .value
    .workspaces
    .iter()
    .filter(|config| {
      config.bind_to_monitor.is_some_and(|monitor_index| {
        monitor.index() == *monitor_index as usize
      })
    })
    .collect::<Vec<_>>();

  // Activate a workspace on the newly added monitor.
  match bound_workspace_configs.len() {
    0 => {
      activate_workspace(None, Some(monitor), state, config)?;
    }
    _ => {
      let workspace_config = bound_workspace_configs.first().unwrap();

      // TODO: Move bound workspaces that are not on the newly added
      // monitor.

      // TODO: Activate all `keep_alive` workspaces for this monitor.

      activate_workspace(
        Some(&workspace_config.name),
        Some(monitor),
        state,
        config,
      )?;
    }
  }

  Ok(())
}
