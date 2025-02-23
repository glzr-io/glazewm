use anyhow::Context;
use tracing::info;
use wm_common::WmEvent;

use super::sort_workspaces;
use crate::{
  commands::container::{
    move_container_within_tree, set_focused_descendant,
  },
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

// This swaps the displayed workspace on monitor 1 and monitor 2.
//
// By default, focus does not change. However, if `stay_on_monitor` is enable,
// the focus with stay on the same monitor.
pub fn swap_workspace(
  monitor_1_index: usize,
  monitor_2_index: usize,
  stay_on_monitor: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("swap_workspace");
  info!("stay_on_monitor: {stay_on_monitor}");

  let focused_workspace = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .context("No workspace is currently focused.")?;

  let focused_monitor = focused_workspace
    .monitor()
    .context("Workspace has no monitor")?;

  let monitors = state.monitors();
  let monitor_1 = monitors.get(monitor_1_index).with_context(|| {
    format!("Monitor at {monitor_1_index} does not exist.")
  })?;

  let monitor_2 = monitors.get(monitor_2_index).with_context(|| {
    format!("Monitor at {monitor_2_index} does not exist.")
  })?;

  let workspace_at_1 = monitor_1
    .displayed_workspace()
    .context("No displayed workspace.")?;

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

  sort_workspaces(monitor_1, config)?;
  sort_workspaces(monitor_2, config)?;

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

  if stay_on_monitor {
    // Flipped because the workspaces just got swap.
    let displayed = if focused_monitor.id() == monitor_1.id() {
      &workspace_at_2
    } else {
      &workspace_at_1
    };

    let container_to_focus = displayed
      .descendant_focus_order()
      .next()
      .unwrap_or_else(|| displayed.clone().into());
    set_focused_descendant(&container_to_focus, None);
  } else {
    let container_to_focus = focused_workspace
      .descendant_focus_order()
      .next()
      .unwrap_or_else(|| focused_workspace.clone().into());
    set_focused_descendant(&container_to_focus, None);
  }

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
