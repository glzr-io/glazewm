use anyhow::Context;
use tracing::info;
use wm_platform::{NativeWindow, WindowId};

use crate::{
  commands::{window::unmanage_window, workspace::deactivate_workspace},
  traits::{CommonGetters, WindowGetters},
  wm_state::WmState,
};

pub fn handle_window_destroyed(
  native_window_id: WindowId,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let found_window = state
    .windows()
    .into_iter()
    .find(|window| window.native().id() == native_window_id);

  // Unmanage the window if it's currently managed.
  if let Some(window) = found_window {
    let workspace = window.workspace().context("No workspace.")?;

    info!("Window closed: {window}");
    unmanage_window(window, state)?;

    // Destroy parent workspace if window was killed while its workspace
    // was not displayed (e.g. via task manager).
    if !workspace.config().keep_alive
      && !workspace.has_children()
      && !workspace.is_displayed()
    {
      deactivate_workspace(workspace, state)?;
    }
  }

  Ok(())
}
