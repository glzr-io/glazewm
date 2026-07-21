use anyhow::Context;
use tracing::info;
use wm_platform::WindowId;

use crate::{
  commands::{
    window::unmanage_window,
    workspace::{
      deactivate_workspace, reapply_assigned_columns,
      workspace_center_window_id,
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

    info!("Window closed: {window}");
    unmanage_window(window, state)?;

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
