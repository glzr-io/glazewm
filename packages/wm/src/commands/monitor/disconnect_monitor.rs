use std::time::{Duration, Instant};

use tracing::info;

use crate::{
  commands::monitor::remove_monitor,
  models::Monitor,
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::{DisconnectedMonitor, WmState},
};

const GHOST_TTL: Duration = Duration::from_secs(3600);

/// Records a monitor's identity and workspaces before removing it from
/// the container tree. This allows workspace restoration when the same
/// physical monitor reconnects.
pub fn disconnect_monitor(
  monitor: Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let props = monitor.native_properties();
  let device_path = props.device_path.clone();
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

  // Remove any existing ghost with same device_path to prevent
  // duplicates.
  if let Some(ref path) = device_path {
    state
      .disconnected_monitors
      .retain(|dm| dm.device_path.as_ref() != Some(path));
  }

  // Prune ghosts older than the TTL.
  let now = Instant::now();
  state
    .disconnected_monitors
    .retain(|dm| now.duration_since(dm.disconnected_at) < GHOST_TTL);

  // Push new ghost record.
  state.disconnected_monitors.push(DisconnectedMonitor {
    device_path,
    hardware_id,
    device_name,
    workspace_names,
    disconnected_at: now,
  });

  // Delegate to existing remove_monitor to detach and redistribute.
  remove_monitor(monitor, state, config)
}
