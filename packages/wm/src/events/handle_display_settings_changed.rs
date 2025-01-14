use anyhow::Context;
use tracing::info;
use wm_platform::Platform;

use crate::{
  commands::monitor::{
    add_monitor, remove_monitor, sort_monitors, update_monitor,
  },
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_display_settings_changed(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Display settings changed.");

  let native_monitors = Platform::sorted_monitors()?;

  let hardware_ids = native_monitors
    .iter()
    .filter_map(|m| m.hardware_id().ok())
    .flatten()
    .cloned()
    .collect::<Vec<_>>();

  let mut pending_monitors = state.monitors();
  let mut new_native_monitors = Vec::new();

  for native_monitor in native_monitors {
    // Get the corresponding `Monitor` instance by either its handle,
    // device path, or hardware ID. Monitor handles and device paths
    // *should* be unique, but can both change over time. The hardware ID
    // is not guaranteed to be unique, so we match against that last.
    let found_monitor = pending_monitors
      .iter()
      .find_map(|monitor| {
        if monitor.native().handle == native_monitor.handle {
          return Some(monitor);
        }

        if monitor.native().device_path().ok()??
          == native_monitor.device_path().ok()??
        {
          return Some(monitor);
        }

        let hardware_id = monitor.native().hardware_id().ok()??.clone();

        // Check that there aren't multiple monitors with the same
        // hardware ID.
        let is_unique =
          hardware_ids.iter().filter(|&id| *id == hardware_id).count()
            == 1;

        if is_unique
          && hardware_id == *native_monitor.hardware_id().ok()??
        {
          Some(monitor)
        } else {
          None
        }
      })
      .cloned();

    match found_monitor {
      Some(found_monitor) => {
        // Remove the monitor from the pending list so that we don't
        // consider it again in the next iteration.
        if let Some(index) = pending_monitors
          .iter()
          .position(|m| m.id() == found_monitor.id())
        {
          pending_monitors.remove(index);
        }

        update_monitor(&found_monitor, native_monitor, state)?;
      }
      None => {
        new_native_monitors.push(native_monitor);
      }
    }
  }

  for native_monitor in new_native_monitors {
    match pending_monitors.first() {
      Some(_) => {
        let monitor = pending_monitors.remove(0);
        update_monitor(&monitor, native_monitor, state)
      }
      // Add monitor if it doesn't exist in state.
      None => add_monitor(native_monitor, state, config),
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
    .containers_to_redraw
    .push(state.root_container.clone().into());

  Ok(())
}
