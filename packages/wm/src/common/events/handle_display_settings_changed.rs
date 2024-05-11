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
    .cloned()
    .collect::<Vec<_>>();

  let monitors = state.monitors();
  let mut new_native_monitors = Vec::new();

  for native_monitor in native_monitors {
    // The `monitor_from_native` function uses the monitor's handle to
    // find the corresponding monitor. Since the monitor handle can change
    // after a display setting change, we also look for a matching device
    // path or hardware ID. The hardware ID is not guaranteed to be unique,
    // so we match against that last.
    let found_monitor =
      state.monitor_from_native(&native_monitor).or_else(|| {
        monitors
          .iter()
          .find_map(|monitor| {
            if monitor.native().device_path().ok()??
              == native_monitor.device_path().ok()??
            {
              return Some(monitor);
            }

            let hardware_id =
              monitor.native().hardware_id().ok()??.clone();

            // Check that there aren't multiple monitors with the same
            // hardware ID.
            let is_unique =
              hardware_ids.iter().filter(|&id| *id == hardware_id).count()
                == 1;

            match is_unique
              && hardware_id == *native_monitor.hardware_id().ok()??
            {
              true => Some(monitor),
              false => None,
            }
          })
          .cloned()
      });

    match found_monitor {
      Some(found_monitor) => {
        info!(
          "Monitor: {:?} {} {:?} {:?}",
          native_monitor.handle,
          native_monitor.device_name()?,
          native_monitor.device_path()?,
          native_monitor.hardware_id()?
        );

        found_monitor.set_native(native_monitor);

        // TODO: Check that a property on the monitor actually changed.
        state.emit_event(WmEvent::MonitorUpdated {
          updated_monitor: found_monitor.clone(),
        });
      }
      None => {
        new_native_monitors.push(native_monitor);
      }
    }
  }
  info!("-------");

  Ok(())
}
