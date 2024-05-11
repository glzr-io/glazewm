use anyhow::Context;
use tracing::info;

use crate::{
  common::platform::{NativeMonitor, Platform},
  containers::{
    commands::{detach_container, move_container_within_tree},
    traits::{CommonGetters, PositionGetters},
  },
  monitors::{commands::add_monitor, Monitor},
  user_config::UserConfig,
  windows::traits::WindowGetters,
  wm_event::WmEvent,
  wm_state::WmState,
};

pub fn handle_display_settings_changed(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Display settings changed.");

  // TODO: Sort `new_native_monitors` by position.
  let native_monitors = Platform::monitors()?;
  let hardware_ids = native_monitors
    .iter()
    .filter_map(|m| m.hardware_id().ok())
    .flatten()
    .cloned()
    .collect::<Vec<_>>();

  let mut pending_monitors = state.monitors();
  let mut new_native_monitors = Vec::new();

  for native_monitor in native_monitors {
    // The `monitor_from_native` function uses the monitor's handle to
    // find the corresponding monitor. Since the monitor handle can change
    // after a display setting change, we also look for a matching device
    // path or hardware ID. The hardware ID is not guaranteed to be unique,
    // so we match against that last.
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

        match is_unique
          && hardware_id == *native_monitor.hardware_id().ok()??
        {
          true => Some(monitor),
          false => None,
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

        update_monitor(found_monitor, native_monitor, state)?;
      }
      None => {
        new_native_monitors.push(native_monitor);
      }
    }
  }

  for native_monitor in new_native_monitors {
    match pending_monitors.get(0) {
      Some(_) => {
        let monitor = pending_monitors.remove(0);
        update_monitor(monitor, native_monitor, state)
      }
      // Add monitor if it doesn't exist in state.
      None => add_monitor(native_monitor, state, config),
    }?;
  }

  // Remove any monitors that no longer exist and move their workspaces
  // to other monitors.
  for pending_monitor in pending_monitors {
    remove_monitor(pending_monitor, state)?;
  }

  for window in state.windows() {
    // Display setting changes can spread windows out sporadically, so mark
    // all windows as needing a DPI adjustment (just in case).
    window.set_has_pending_dpi_adjustment(true);

    // Need to update floating position of moved windows when a monitor is
    // disconnected or if the primary display is changed. The primary
    // display dictates the position of 0,0.
    let workspace = window.workspace().context("No workspace")?;
    window.set_floating_placement(
      window
        .floating_placement()
        .translate_to_center(&workspace.to_rect()?),
    );
  }

  // Redraw full container tree.
  state
    .containers_to_redraw
    .push(state.root_container.clone().into());

  Ok(())
}

fn update_monitor(
  monitor: Monitor,
  native_monitor: NativeMonitor,
  state: &mut WmState,
) -> anyhow::Result<()> {
  info!(
    "Monitor: {:?} {} {:?} {:?}",
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

fn remove_monitor(
  monitor: Monitor,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let target_monitor = state
    .monitors()
    .into_iter()
    .find(|m| m.id() != monitor.id())
    .context("No target monitor to move workspaces.")?;

  // Avoid moving empty workspaces.
  let workspaces_to_move = monitor
    .children()
    .into_iter()
    .filter_map(|container| container.as_workspace().cloned())
    .filter(|workspace| {
      workspace.has_children() || workspace.config().keep_alive
    });

  for workspace in workspaces_to_move {
    // Move workspace to target monitor.
    move_container_within_tree(
      workspace.clone().into(),
      target_monitor.clone().into(),
      target_monitor.child_count(),
    )?;

    state.emit_event(WmEvent::WorkspaceMoved {
      workspace: workspace.clone(),
      new_monitor: target_monitor.clone(),
    });
  }

  detach_container(monitor.clone().into())?;

  state.emit_event(WmEvent::MonitorRemoved {
    removed_id: monitor.id(),
    removed_device_name: monitor.name()?.to_string(),
  });

  Ok(())
}
