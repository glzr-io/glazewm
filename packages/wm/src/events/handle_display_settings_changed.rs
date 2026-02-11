use anyhow::Context;
use tracing::info;
#[cfg(target_os = "windows")]
use wm_platform::{DisplayDeviceExtWindows, DisplayExtWindows};

use crate::{
  commands::monitor::{
    add_monitor, remove_monitor, sort_monitors, update_monitor,
  },
  models::NativeMonitorProperties,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_display_settings_changed(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Display settings changed.");

  let displays = state.dispatcher.displays()?;

  let mut pending_monitors = state.monitors();
  let mut new_native_displays = Vec::new();

  for display in displays {
    let properties = NativeMonitorProperties::try_from(&display)?;

    // Get the corresponding `Monitor` instance by either its handle,
    // device path, or hardware ID. Monitor handles and device paths
    // *should* be unique, but can both change over time. The hardware ID
    // is not guaranteed to be unique, so we match against that last.
    let found_monitor = pending_monitors
      .iter()
      .find(|monitor| {
        let monitor_properties = monitor.native_properties();

        #[cfg(target_os = "macos")]
        {
          monitor_properties.id == properties.id
        }
        #[cfg(target_os = "windows")]
        {
          monitor_properties.handle == properties.handle
            || monitor_properties
              .device_path
              .is_some_and(|p| p == properties.device_path())
            || monitor_properties.hardware_id.is_some_and(|p| {
              // Check that there aren't multiple monitors with the same
              // hardware ID.
              let is_unique = pending_monitors
                .iter()
                .filter(|o| o.native_properties().hardware_id == p)
                .count()
                == 1;

              is_unique && p == properties.hardware_id
            })
        }
      })
      .cloned();

    match found_monitor {
      Some(found_monitor) => {
        // Remove from pending so it's not considered again.
        if let Some(index) = pending_monitors
          .iter()
          .position(|m| m.id() == found_monitor.id())
        {
          pending_monitors.remove(index);
        }

        update_monitor(&found_monitor, display, state)?;
      }
      None => {
        new_native_displays.push(display);
      }
    }
  }

  // Pair unmatched displays with unmatched monitors, or add new ones.
  for native_display in new_native_displays {
    match pending_monitors.first() {
      Some(_) => {
        let monitor = pending_monitors.remove(0);
        update_monitor(&monitor, native_display, state)
      }
      // Add monitor if it doesn't exist in state.
      None => {
        let native_properties =
          NativeMonitorProperties::try_from(&native_display)?;
        add_monitor(native_display, native_properties, state, config)
      }
    }?;
  }

  // Remove any monitors that no longer exist and move their workspaces
  // to other monitors.
  //
  // Prevent removal of the last monitor (i.e. for when all monitors are
  // disconnected). This will cause the WM's monitors to mismatch the OS
  // monitor state, however, it'll be updated correctly when a new monitor
  // is connected again.
  for pending_monitor in pending_monitors {
    if state.monitors().len() != 1 {
      remove_monitor(pending_monitor, state, config)?;
    }
  }

  // Sort monitors by position.
  sort_monitors(&state.root_container)?;

  for window in state.windows() {
    // Display setting changes can spread windows out sporadically, so mark
    // all windows as needing a DPI adjustment (just in case).
    window.set_has_pending_dpi_adjustment(true);

    // Need to update floating position of moved windows when a monitor is
    // disconnected or if the primary display is changed. The primary
    // display dictates the position of 0,0.
    let workspace = window.workspace().context("No workspace.")?;
    window.set_floating_placement(
      window
        .floating_placement()
        .translate_to_center(&workspace.to_rect()?),
    );
  }

  // Redraw full container tree.
  state
    .pending_sync
    .queue_container_to_redraw(state.root_container.clone());

  Ok(())
}
