use tracing::info;

use crate::{
  common::platform::NativeMonitor, monitors::Monitor, wm_event::WmEvent,
  wm_state::WmState,
};

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
    updated_monitor: monitor,
  });

  Ok(())
}
