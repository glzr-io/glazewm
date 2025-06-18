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

  let bound_workspace_configs = config.keep_alive_workspace_configs();

  // Activate all `keep_alive` workspaces for this monitor.
  for workspace_config in bound_workspace_configs {
    #[allow(clippy::cast_possible_truncation)]
    if workspace_config.bind_to_monitor == Some(monitor.index() as u32) {
      activate_workspace(
        Some(&workspace_config.name),
        Some(monitor.clone()),
        state,
        config,
      )?;
    }
  }

  // Activate a workspace on the newly added monitor.
  activate_workspace(None, Some(monitor), state, config)
    .context("At least 1 workspace is required per monitor.")?;

  Ok(())
}
