use tracing::info;

use crate::{
  common::platform::Platform, user_config::UserConfig, wm_event::WmEvent,
  wm_state::WmState,
};

pub fn handle_display_settings_changed(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Display settings changed.");

  let native_monitors = Platform::monitors()?;
  let hardware_ids = native_monitors
    .iter()
    .filter_map(|m| m.hardware_id().ok())
    .flatten()
    .collect::<Vec<_>>();

  let monitors = state.monitors();

  for native_monitor in native_monitors {
    // Hardware ID is not guaranteed to be unique, so we check
    // for that as a fallback.
    let found_monitor = monitors.iter().find(|monitor| {
      monitor.native().handle == native_monitor.handle
        || monitor.native().device_path().is_ok_and(|path| {
          path.is_some_and(|path| {
            Some(Some(path)) == native_monitor.device_path().ok()
          })
        })
        // TODO: Check that there aren't multiple monitors with the same
        // hardware ID.
        || monitor.native().hardware_id().is_ok_and(|id| {
          id.is_some_and(|id| {
            Some(Some(id)) == native_monitor.hardware_id().ok()
          })
        })
      // == native_monitor.hardware_id().ok() && !hardware_ids.contains(monitor.native().hardware_id().unwrap()))
    });

    if let Some(found_monitor) = found_monitor {
      info!(
        "Monitor: {:?} {} {:?} {:?}",
        native_monitor.handle,
        native_monitor.device_name()?,
        native_monitor.device_path()?,
        native_monitor.hardware_id()?
      );

      // found_monitor.set_native(native_monitor);

      state.emit_event(WmEvent::MonitorUpdated {
        updated_monitor: found_monitor.clone(),
      })
    }
  }
  info!("-------");

  Ok(())
}
