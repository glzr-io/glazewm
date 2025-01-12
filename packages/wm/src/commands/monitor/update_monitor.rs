use tracing::info;
use wm_common::WmEvent;
use wm_platform::NativeMonitor;

use crate::{models::Monitor, wm_state::WmState};

pub fn update_monitor(
  monitor: Monitor,
  native_monitor: NativeMonitor,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // TODO: Add monitor display trait.
  info!(
    "Updating monitor: {:?} {} {:?} {:?}",
    native_monitor.handle,
    native_monitor.device_name()?,
    native_monitor.device_path()?,
    native_monitor.hardware_id()?
  );

  monitor.set_native(native_monitor);

  // TODO: Check that a property on the monitor actually changed.
  state.emit_event(WmEvent::MonitorUpdated {
    updated_monitor: monitor.to_dto()?,
  });

  Ok(())
}
