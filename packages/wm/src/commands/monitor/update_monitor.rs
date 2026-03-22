use tracing::info;
use wm_common::WmEvent;
use wm_platform::Display;

use crate::{
  models::{Monitor, NativeMonitorProperties},
  wm_state::WmState,
};

pub fn update_monitor(
  monitor: &Monitor,
  native_display: &Display,
  native_properties: NativeMonitorProperties,
  state: &mut WmState,
) -> anyhow::Result<()> {
  monitor.set_native(native_display.clone());
  monitor.set_native_properties(native_properties);

  info!("Monitor updated: {monitor}");

  // TODO: Check that a property on the monitor actually changed.
  state.emit_event(WmEvent::MonitorUpdated {
    updated_monitor: monitor.to_dto()?,
  });

  Ok(())
}
