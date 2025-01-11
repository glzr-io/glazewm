use anyhow::Context;
use tracing::info;
use wm_platform::NativeWindow;

use crate::{
  containers::traits::CommonGetters, windows::commands::unmanage_window,
  wm_state::WmState, workspaces::commands::deactivate_workspace,
};

pub fn handle_window_destroyed(
  native_window: NativeWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  // Unmanage the window if it's currently managed.
  if let Some(window) = found_window {
    // TODO: Log window details.
    let workspace = window.workspace().context("No workspace.")?;

    info!("Window closed");
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
