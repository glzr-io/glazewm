use anyhow::Context;
use wm_common::try_warn;

use crate::{
  commands::{
    monitor::{
      add_monitor, move_bounded_workspaces_to_new_monitor, remove_monitor,
      sort_monitors, update_monitor,
    },
    window::manage_window,
  },
  models::{Monitor, NativeMonitorProperties},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Handles changes to the display configuration (e.g. monitor
/// connect/disconnect, resolution changes, lid close/reopen).
///
/// Updates the monitor and workspace topology to match the new display
/// layout, re-discovers any unmanaged windows, and redraws the full
/// container tree.
pub fn handle_display_settings_changed(
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  tracing::info!("Display settings changed.");

  // Ignore the event if retrieval of the displays fails.
  let displays = try_warn!(state.dispatcher.sorted_displays());
  let mut pending_monitors = state.monitors();
  let mut unmatched_displays = Vec::new();

  // Match each display to an existing monitor and update it.
  for display in displays {
    // TODO: Create `NativeMonitorProperties` instances for displays just
    // once (created in loop below and in `update_monitor`).
    let properties = NativeMonitorProperties::try_from(&display)?;

    match find_matching_monitor(&pending_monitors, &properties) {
      Some((monitor, index)) => {
        update_monitor(monitor, &display, state)?;
        pending_monitors.remove(index);
      }
      None => unmatched_displays.push(display),
    }
  }

  let mut new_monitors: Vec<Monitor> = Vec::new();

  // Pair unmatched displays with unmatched monitors, or add new ones.
  for display in unmatched_displays {
    if pending_monitors.is_empty() {
      let properties = NativeMonitorProperties::try_from(&display)?;
      let monitor = add_monitor(display, properties, state)?;
      new_monitors.push(monitor);
    } else {
      let monitor = pending_monitors.remove(0);
      update_monitor(&monitor, &display, state)?;
    }
  }

  // Remove monitors that no longer have a corresponding display and move
  // their workspaces to other monitors.
  //
  // Prevent removal of the last monitor (i.e. for when all monitors are
  // disconnected). This will cause the WM's monitors to temporarily
  // mismatch the OS monitor state, however, it'll be updated correctly
  // when a new monitor is connected again.
  for monitor in pending_monitors {
    if state.monitors().len() > 1 {
      remove_monitor(monitor, state, config)?;
    }
  }

  // Sort monitors by position.
  sort_monitors(&state.root_container)?;

  for new_monitor in new_monitors {
    move_bounded_workspaces_to_new_monitor(&new_monitor, state, config)?;
  }

  // Re-enumerate visible windows and manage any that are not already
  // tracked. This handles cases where windows fall out of the managed
  // state during display changes (e.g. laptop lid close/reopen cycles).
  match state.dispatcher.visible_windows() {
    Ok(visible_windows) => {
      for native_window in visible_windows.into_iter().rev() {
        if state.window_from_native(&native_window).is_none() {
          let nearest_workspace = state
            .nearest_monitor(&native_window)
            .and_then(|m| m.displayed_workspace());

          if let Some(workspace) = nearest_workspace {
            if let Err(err) = manage_window(
              native_window,
              Some(workspace.into()),
              state,
              config,
            ) {
              tracing::warn!(
                "Failed to manage window during re-sync: {:?}",
                err
              );
            }
          }
        }
      }
    }
    Err(err) => {
      tracing::warn!("Failed to re-enumerate visible windows: {:?}", err);
    }
  }

  for window in state.windows() {
    // Display setting changes can spread windows out sporadically, so mark
    // all windows as needing a DPI adjustment (just in case).
    window.set_has_pending_dpi_adjustment(true);

    // Need to update floating position of moved windows when a monitor is
    // disconnected or if the primary display is changed. The primary
    // display dictates the position of 0,0.
    let workspace = window.workspace().context("No workspace.")?;

    let should_recenter = if window.has_custom_floating_placement() {
      let workspace_rect = workspace.to_rect()?;

      // Keep the placement if it still intersects the workspace, since
      // `PlatformEvent::DisplaySettingsChanged` can be triggered by
      // non-monitor changes (e.g. unplugging a USB device).
      window
        .floating_placement()
        .intersection_area(&workspace_rect)
        == 0
    } else {
      true
    };

    if should_recenter {
      window.set_floating_placement(
        window
          .floating_placement()
          .translate_to_center(&workspace.to_rect()?),
      );
    }
  }

  // Redraw full container tree.
  state
    .pending_sync
    .queue_container_to_redraw(state.root_container.clone());

  Ok(())
}

/// Finds the monitor matching the given display properties.
///
/// Returns the monitor and its index within the list of monitors.
fn find_matching_monitor<'a>(
  monitors: &'a [Monitor],
  properties: &NativeMonitorProperties,
) -> Option<(&'a Monitor, usize)> {
  monitors.iter().enumerate().find_map(|(index, monitor)| {
    let existing = monitor.native_properties();

    let is_match = {
      #[cfg(target_os = "macos")]
      {
        existing.device_uuid == properties.device_uuid
      }

      // On Windows, match the monitor by:
      // 1. Its handle
      // 2. Its device path
      // 3. Its hardware ID (if unique)
      //
      // Monitor handles and device paths are unique, but can change over
      // time. The hardware ID is not guaranteed to be unique, so we
      // match against that last.
      #[cfg(target_os = "windows")]
      {
        existing.handle == properties.handle
          || existing.device_path.as_deref().is_some_and(|device_path| {
            properties.device_path.as_deref() == Some(device_path)
          })
          || existing.hardware_id.as_deref().is_some_and(|hardware_id| {
            let is_unique = monitors
              .iter()
              .filter(|other_monitor| {
                other_monitor.native_properties().hardware_id.as_deref()
                  == Some(hardware_id)
              })
              .count()
              == 1;

            is_unique
              && properties.hardware_id.as_deref() == Some(hardware_id)
          })
      }
    };

    is_match.then_some((monitor, index))
  })
}
