use anyhow::Context;
use tracing::info;
use wm_common::TilingStrategy;
use wm_platform::NativeWindow;

use crate::{
  commands::{window::unmanage_window, workspace::deactivate_workspace},
  traits::CommonGetters,
  wm_state::WmState,
};

pub fn handle_window_destroyed(
  native_window: &NativeWindow,
  state: &mut WmState,
  tiling_strategy: &TilingStrategy,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);

  // Unmanage the window if it's currently managed.
  if let Some(window) = found_window {
    let workspace = window.workspace().context("No workspace.")?;

    info!("Window closed: {window}");
    unmanage_window(window, state, tiling_strategy)?;

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
