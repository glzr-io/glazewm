use anyhow::Context;

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

use super::activate_workspace;

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
      .descendants()
      .filter_map(|descendant| descendant.as_window_container().ok());

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

    // Prevent original monitor from having no workspaces.
    if monitor.child_count() == 0 {
      activate_workspace(None, &monitor, state, config)?;
    }

    state.emit_event(WmEvent::WorkspaceMoved {
      workspace: workspace.to_dto()?,
      new_monitor: target_monitor.to_dto()?,
    });
  }

  Ok(())
}
