use anyhow::Context;

use super::{activate_workspace, sort_workspaces};
use crate::{
  common::Direction,
  containers::{
    commands::move_container_within_tree,
    traits::{CommonGetters, PositionGetters},
    WindowContainer,
  },
  user_config::UserConfig,
  windows::traits::WindowGetters,
  wm_event::WmEvent,
  wm_state::WmState,
  workspaces::Workspace,
};

pub fn move_workspace_in_direction(
  workspace: Workspace,
  direction: Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let monitor = workspace.monitor().context("No monitor.")?;
  let target_monitor = state.monitor_in_direction(&monitor, &direction)?;

  if let Some(target_monitor) = target_monitor {
    move_container_within_tree(
      workspace.clone().into(),
      target_monitor.clone().into(),
      target_monitor.child_count(),
      state,
    )?;

    let windows = workspace
      .descendants_of_type::<WindowContainer>();

    for window in windows {
      window.set_has_pending_dpi_adjustment(true);

      window.set_floating_placement(
        window
          .floating_placement()
          .translate_to_center(&workspace.to_rect()?),
      );

      if let WindowContainer::NonTilingWindow(window) = &window {
        window.set_insertion_target(None);
      }
    }

    state
      .pending_sync
      .containers_to_redraw
      .push(workspace.clone().into());

    // Prevent original monitor from having no workspaces.
    if monitor.child_count() == 0 {
      activate_workspace(None, &monitor, state, config)?;
    }

    sort_workspaces(target_monitor, config)?;

    state.emit_event(WmEvent::WorkspaceUpdated {
      updated_workspace: workspace.to_dto()?,
    });
  }

  Ok(())
}
