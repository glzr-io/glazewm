use anyhow::Context;
use tracing::info;

use crate::{
  containers::{
    commands::{move_container_within_tree, set_focused_descendant},
    traits::{CommonGetters, PositionGetters},
    WindowContainer,
  },
  user_config::UserConfig,
  windows::{traits::WindowGetters, WindowState},
  wm_state::WmState,
  workspaces::commands::activate_workspace,
};

pub fn move_window_to_workspace(
  window: WindowContainer,
  workspace_name: &str,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  info!("Moving window to workspace: {:?}.", workspace_name);

  let current_workspace = window.workspace().context("No workspace.")?;
  let current_monitor =
    current_workspace.monitor().context("No monitor.")?;

  // Retrieve or activate a workspace by its name.
  let target_workspace = match state.workspace_by_name(&workspace_name) {
    Some(workspace) => Some(workspace),
    None => {
      activate_workspace(
        Some(&workspace_name),
        &current_monitor,
        state,
        config,
      )?;

      state.workspace_by_name(&workspace_name)
    }
  };

  if let Some(target_workspace) = target_workspace {
    if target_workspace.id() == current_workspace.id() {
      return Ok(());
    }

    let target_monitor =
      target_workspace.monitor().context("No monitor.")?;

    // Since target workspace could be on a different monitor, adjustments
    // might need to be made because of DPI.
    if current_monitor
      .has_dpi_difference(&target_monitor.clone().into())?
    {
      window.set_has_pending_dpi_adjustment(true);
    }

    // Update floating placement if the window has to cross monitors.
    if target_monitor.id() != current_monitor.id() {
      window.set_floating_placement(
        window
          .floating_placement()
          .translate_to_center(&target_workspace.to_rect()?),
      );
    }

    let focus_target = state.focus_target_after_removal(&window);

    // Since the workspace that gets displayed is the last focused child,
    // focus needs to be reassigned to the displayed workspace.
    let focus_reset_target = match target_workspace.is_displayed() {
      true => None,
      false => target_monitor.descendant_focus_order().next(),
    };

    let insertion_sibling = target_workspace
      .descendant_focus_order()
      .filter_map(|descendant| descendant.as_window_container().ok())
      .find(|descendant| descendant.state() == WindowState::Tiling);

    // Insert the window into the target workspace.
    match (window.is_tiling_window(), insertion_sibling.is_some()) {
      (true, true) => {
        if let Some(insertion_sibling) = insertion_sibling {
          move_container_within_tree(
            window.clone().into(),
            insertion_sibling.clone().parent().context("No parent.")?,
            insertion_sibling.index() + 1,
            state,
          )?;
        }
      }
      _ => {
        move_container_within_tree(
          window.clone().into(),
          target_workspace.clone().into(),
          target_workspace.child_count(),
          state,
        )?;
      }
    }

    if let Some(focus_reset_target) = focus_reset_target {
      set_focused_descendant(focus_reset_target, None);
      state.pending_sync.focus_change = true;
    }

    if let Some(focus_target) = focus_target {
      set_focused_descendant(focus_target, None);
      state.pending_sync.focus_change = true;
    }

    match window {
      WindowContainer::NonTilingWindow(_) => {
        state.pending_sync.containers_to_redraw.push(window.into());
      }
      WindowContainer::TilingWindow(_) => {
        state
          .pending_sync
          .containers_to_redraw
          .extend(current_workspace.tiling_children().map(Into::into));
      }
    }
  }

  Ok(())
}
