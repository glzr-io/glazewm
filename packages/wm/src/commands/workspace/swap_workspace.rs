use anyhow::Context;
use tracing::info;
use wm_common::WmEvent;

use super::sort_workspaces;
use crate::{
  commands::container::{
    move_container_within_tree, set_focused_descendant,
  },
  models::{Container, MonitorTarget},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// This swaps the displayed workspace on monitor 1 and monitor 2.
pub fn swap_workspace_explicit(
  monitor_1_index: usize,
  monitor_2_index: usize,
  change_focus: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let monitors = state.monitors();

  let monitor_1 = monitors.get(monitor_1_index).with_context(|| {
    format!("Monitor at {monitor_1_index} does not exist.")
  })?;

  let monitor_2 = monitors.get(monitor_2_index).with_context(|| {
    format!("Monitor at {monitor_2_index} does not exist.")
  })?;

  swap_workspace_internal(
    &monitor_1.as_container(),
    &monitor_2.as_container(),
    change_focus,
    state,
    config,
  )
}

/// This swap the current focused workspace with the one displayed at
/// `target_monitor_index`.
pub fn swap_workspace(
  target: MonitorTarget,
  change_focus: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_monitor = state
    .focused_container()
    .and_then(|container| container.monitor())
    .context("No monitor is focused.")?;

  let target_monitor = match target {
    MonitorTarget::Direction(direction) => {
      state.monitor_in_direction(&focused_monitor, &direction)?
    }
    MonitorTarget::Index(index) => {
      let monitors = state.monitors();
      monitors.get(index).cloned()
    }
    MonitorTarget::Monitor(monitor) => Some(monitor),
  };

  if let Some(target_monitor) = target_monitor {
    swap_workspace_internal(
      &focused_monitor.into(),
      &target_monitor.into(),
      change_focus,
      state,
      config,
    )?;
  }

  Ok(())
}

/// This swaps the displayed workspace on `container_1` and `container_2`.
///
/// If one of the workspace moved is in focus, by default the focus will
/// follow the swap, however, if `stay_on_monitor` is set to true, focus
/// if be change to the swapped workspace. This is to not change monitor
/// focus.
///
/// Otherwise, `stay_on_monitor` will do nothing.
pub fn swap_workspace_internal(
  container_1: &Container,
  container_2: &Container,
  change_focus: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("swap_workspace");
  info!("change_focus: {change_focus}");

  let focused_workspace = state
    .focused_container()
    .and_then(|container| container.workspace())
    .context("No focused workspace")?;

  let monitor_1 = container_1
    .monitor()
    .context("container_1 has no monitor.")?;

  let workspace_at_1 = monitor_1
    .displayed_workspace()
    .context("No displayed workspace.")?;

  let monitor_2 = container_2
    .monitor()
    .context("container_2 has no monitor.")?;

  let workspace_at_2 = monitor_2
    .displayed_workspace()
    .context("No displayed workspace.")?;

  info!("monitor_1: {monitor_1}");
  info!("workspace_at_1: {workspace_at_1}");
  info!("monitor_2: {monitor_2}");
  info!("workspace_at_2: {workspace_at_2}");

  move_container_within_tree(
    &workspace_at_1.clone().into(),
    &monitor_2.clone().into(),
    monitor_2.child_count(),
    state,
  )?;

  move_container_within_tree(
    &workspace_at_2.clone().into(),
    &monitor_1.clone().into(),
    monitor_1.child_count(),
    state,
  )?;

  sort_workspaces(&monitor_1, config)?;
  sort_workspaces(&monitor_2, config)?;

  let windows = workspace_at_1
    .descendants()
    .filter_map(|descendant| descendant.as_window_container().ok());

  for window in windows {
    window.set_has_pending_dpi_adjustment(true);

    window.set_floating_placement(
      window
        .floating_placement()
        .translate_to_center(&workspace_at_1.to_rect()?),
    );
  }

  let windows = workspace_at_2
    .descendants()
    .filter_map(|descendant| descendant.as_window_container().ok());

  for window in windows {
    window.set_has_pending_dpi_adjustment(true);

    window.set_floating_placement(
      window
        .floating_placement()
        .translate_to_center(&workspace_at_2.to_rect()?),
    );
  }

  let workspace_to_focus = if focused_workspace.id() == workspace_at_1.id()
  {
    if change_focus {
      &workspace_at_2
    } else {
      &workspace_at_1
    }
  } else if focused_workspace.id() == workspace_at_2.id() {
    if change_focus {
      &workspace_at_1
    } else {
      &workspace_at_2
    }
  } else {
    // There is nothing else to focus to so default back to the orignal
    // focus.
    &focused_workspace
  };

  let container_to_focus = workspace_to_focus
    .descendant_focus_order()
    .next()
    .unwrap_or_else(|| workspace_to_focus.clone().as_container());

  set_focused_descendant(&container_to_focus, None);

  state
    .pending_sync
    .queue_focus_change()
    .queue_container_to_redraw(focused_workspace)
    .queue_container_to_redraw(workspace_at_1.clone())
    .queue_container_to_redraw(workspace_at_2.clone())
    .queue_cursor_jump();

  state.emit_event(WmEvent::WorkspaceUpdated {
    updated_workspace: workspace_at_1.to_dto()?,
  });

  state.emit_event(WmEvent::WorkspaceUpdated {
    updated_workspace: workspace_at_2.to_dto()?,
  });

  Ok(())
}
