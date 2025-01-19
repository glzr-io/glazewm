use tracing::info;
use wm_common::WmEvent;
use wm_platform::NativeMonitor;

use crate::{models::Monitor, wm_state::WmState};

pub fn update_monitor(
  monitor: &Monitor,
  native_monitor: NativeMonitor,
  state: &mut WmState,
) -> anyhow::Result<()> {
  info!("Updating monitor: {monitor}");

  monitor.set_native(native_monitor);

  // TODO: Check that a property on the monitor actually changed.
  state.emit_event(WmEvent::MonitorUpdated {
    updated_monitor: monitor.to_dto()?,
  });

  Ok(())
}
