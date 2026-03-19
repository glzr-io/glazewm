use std::time::{Duration, Instant};

use tracing::info;

use crate::{
  commands::monitor::remove_monitor,
  models::Monitor,
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::{DisconnectedMonitor, WmState},
};

const GHOST_TTL: Duration = Duration::from_hours(1);

/// Records a monitor's identity and workspaces before removing it from
/// the container tree. This allows workspace restoration when the same
/// physical monitor reconnects.
pub fn disconnect_monitor(
  monitor: Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let props = monitor.native_properties();
  #[cfg(target_os = "macos")]
  let device_uuid = props.device_uuid.clone();
  #[cfg(target_os = "windows")]
  let device_path = props.device_path.clone();
  #[cfg(target_os = "windows")]
  let hardware_id = props.hardware_id.clone();
  let device_name = props.device_name.clone();

  // Collect workspace names ordered by focus (displayed workspace first).
  let workspace_names: Vec<String> = monitor
    .child_focus_order()
    .filter_map(|c| c.as_workspace().cloned())
    .map(|ws| ws.config().name)
    .collect();

  info!(
    "Recording disconnected monitor '{}' with workspaces: {:?}",
    device_name, workspace_names
  );

  // Remove any existing ghost for the same physical monitor to prevent
  // duplicates.
  state.disconnected_monitors.retain(|dm| {
    #[cfg(target_os = "macos")]
    {
      dm.device_uuid != device_uuid
    }
    #[cfg(target_os = "windows")]
    {
      let path_match = device_path.is_some()
        && dm.device_path.as_ref() == device_path.as_ref();
      let hw_match = hardware_id.is_some()
        && dm.hardware_id.as_ref() == hardware_id.as_ref();
      !path_match && !hw_match
    }
  });

  // Prune ghosts older than the TTL.
  let now = Instant::now();
  state
    .disconnected_monitors
    .retain(|dm| now.duration_since(dm.disconnected_at) < GHOST_TTL);

  // Push new ghost record.
  state.disconnected_monitors.push(DisconnectedMonitor {
    #[cfg(target_os = "macos")]
    device_uuid,
    #[cfg(target_os = "windows")]
    device_path,
    #[cfg(target_os = "windows")]
    hardware_id,
    device_name,
    workspace_names,
    disconnected_at: now,
  });

  // Delegate to existing remove_monitor to detach and redistribute.
  remove_monitor(monitor, state, config)
}
