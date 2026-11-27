use anyhow::Context;
use tracing::info;
use wm_platform::WindowId;

use crate::{
  commands::{
    container::set_focused_descendant,
    window::unmanage_window,
    workspace::{
      column_focus_after_close, deactivate_workspace,
      reapply_assigned_columns, workspace_center_window_id,
    },
  },
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_window_destroyed(
  native_window_id: WindowId,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state
    .windows()
    .into_iter()
    .find(|window| window.native().id() == native_window_id);

  // Unmanage the window if it's currently managed.
  if let Some(window) = found_window {
    let workspace = window.workspace().context("No workspace.")?;

    // Note the current center before unmanaging so the reapply keeps it in
    // place — closing a side window shouldn't move the center window.
    let center = workspace_center_window_id(&workspace);
    let focus_target = column_focus_after_close(&window, state, config)?;

    info!("Window closed: {window}");
    unmanage_window(window, state)?;

    // Focus before the reapply, so a closed center is replaced by the
    // newly focused window.
    if let Some(target) = focus_target {
      set_focused_descendant(&target.into(), None);
      state.pending_sync.queue_focus_change();
    }

    // Destroy parent workspace if window was killed while its workspace
    // was not displayed (e.g. via task manager).
    if !workspace.config().keep_alive
      && !workspace.has_children()
      && !workspace.is_displayed()
    {
      deactivate_workspace(workspace, state)?;
    } else {
      // Re-tidy the workspace's columns (if any) now a window's gone.
      reapply_assigned_columns(&workspace, center, state, config)?;
    }
  }

  Ok(())
}
