use anyhow::Context;
use tracing::info;
use wm_common::WmEvent;
use wm_platform::Display;

use crate::{
  commands::{
    container::{attach_container, move_container_within_tree},
    workspace::{activate_workspace, sort_workspaces},
  },
  models::{Monitor, Workspace},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn add_monitor(
  native_display: Display,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // Create `Monitor` instance. This uses the working area of the monitor
  // instead of the bounds of the display. The working area excludes
  // taskbars and other reserved display space.
  let monitor = Monitor::new(native_display);

  attach_container(
    &monitor.clone().into(),
    &state.root_container.clone().into(),
    None,
  )?;

  info!("Monitor added: {monitor}");

  state.emit_event(WmEvent::MonitorAdded {
    added_monitor: monitor.to_dto()?,
  });

  let bound_workspace_configs = config
    .value
    .workspaces
    .iter()
    .filter(|config| {
      config.bind_to_monitor.is_some_and(|monitor_index| {
        monitor.index() == monitor_index as usize
      })
    })
    .collect::<Vec<_>>();

  for workspace_config in bound_workspace_configs {
    let existing_workspace =
      state.workspace_by_name(&workspace_config.name);

    if let Some(existing_workspace) = existing_workspace {
      // Move workspaces that should be bound to the newly added monitor.
      move_workspace_to_monitor(
        &existing_workspace,
        &monitor,
        state,
        config,
      )?;
    } else if workspace_config.keep_alive {
      // Activate all `keep_alive` workspaces for this monitor.
      activate_workspace(
        Some(&workspace_config.name),
        Some(monitor.clone()),
        state,
        config,
      )?;
    }
  }

  // Make sure the monitor has at least one workspace. This will
  // automatically prioritize bound workspace configs and fall back to the
  // first available one if needed.
  if monitor.child_count() == 0 {
    activate_workspace(None, Some(monitor), state, config)?;
  }

  Ok(())
}

// TODO: Move to its own file once `swap-workspace` PR is merged.
// Ref: https://github.com/glzr-io/glazewm/pull/980.
pub fn move_workspace_to_monitor(
  workspace: &Workspace,
  target_monitor: &Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let origin_monitor = workspace.monitor().context("No monitor.")?;

  // Get currently displayed workspace on the target monitor.
  let displayed_workspace = target_monitor
    .displayed_workspace()
    .context("No displayed workspace.")?;

  move_container_within_tree(
    &workspace.clone().into(),
    &target_monitor.clone().into(),
    target_monitor.child_count(),
    state,
  )?;

  let windows = workspace
    .descendants()
    .filter_map(|descendant| descendant.as_window_container().ok());

  for window in windows {
    window.set_has_pending_dpi_adjustment(true);

    window.set_floating_placement(
      window
        .floating_placement()
        .translate_to_center(&workspace.to_rect()?),
    );
  }

  state
    .pending_sync
    .queue_container_to_redraw(workspace.clone())
    .queue_container_to_redraw(displayed_workspace);

  match origin_monitor.child_count() {
    0 => {
      // Prevent origin monitor from having no workspaces.
      activate_workspace(None, Some(origin_monitor), state, config)?;
    }
    _ => {
      // Redraw the workspace on the origin monitor.
      state.pending_sync.queue_container_to_redraw(
        origin_monitor
          .displayed_workspace()
          .context("No displayed workspace.")?,
      );
    }
  }

  sort_workspaces(target_monitor, config)?;

  state.emit_event(WmEvent::WorkspaceUpdated {
    updated_workspace: workspace.to_dto()?,
  });

  Ok(())
}
