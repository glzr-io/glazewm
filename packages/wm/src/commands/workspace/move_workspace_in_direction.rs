use anyhow::Context;
use wm_common::{Direction, WmEvent};

use super::{activate_workspace, deactivate_workspace, sort_workspaces};
use crate::{
  commands::container::move_container_within_tree,
  models::Workspace,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn move_workspace_in_direction(
  workspace: &Workspace,
  direction: &Direction,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let origin_monitor = workspace.monitor().context("No monitor.")?;
  let target_monitor =
    state.monitor_in_direction(&origin_monitor, direction)?;

  if let Some(target_monitor) = target_monitor {
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

    state.pending_sync.cursor_jump = true;
    state
      .pending_sync
      .containers_to_redraw
      .extend([workspace.clone().into(), displayed_workspace.into()]);

    match origin_monitor.child_count() {
      0 => {
        // Prevent origin monitor from having no workspaces.
        activate_workspace(None, Some(origin_monitor), state, config)?;
      }
      _ => {
        // Redraw the workspace on the origin monitor.
        state.pending_sync.containers_to_redraw.push(
          origin_monitor
            .displayed_workspace()
            .context("No displayed workspace.")?
            .into(),
        );
      }
    }

    // Get empty workspace to destroy (if one is found). Cannot destroy
    // empty workspaces if they're the only workspace on the monitor.
    let workspace_to_destroy =
      target_monitor.workspaces().into_iter().find(|workspace| {
        !workspace.config().keep_alive
          && !workspace.has_children()
          && !workspace.is_displayed()
      });

    if let Some(workspace) = workspace_to_destroy {
      deactivate_workspace(workspace, state)?;
    }

    sort_workspaces(&target_monitor, config)?;

    state.emit_event(WmEvent::WorkspaceUpdated {
      updated_workspace: workspace.to_dto()?,
    });
  }

  Ok(())
}
