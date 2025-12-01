use anyhow::Context;
use tracing::info;
use wm_platform::NativeWindow;

use crate::{
  commands::{
    window::{manage_window::rebuild_spiral_layout, unmanage_window},
    workspace::deactivate_workspace,
  },
  models::TilingWindow,
  traits::CommonGetters,
  wm_state::WmState,
};

pub fn handle_window_destroyed(
  native_window: &NativeWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);

  if let Some(window) = found_window {
    let workspace = window.workspace().context("No workspace.")?;
    let is_tiling = window.is_tiling_window();

    info!("Window closed: {window}");
    unmanage_window(window, state)?;

    // Crucial: Rebuild spiral layout to clean up the tree holes left by detach_container
    if is_tiling {
        let remaining_windows: Vec<TilingWindow> = workspace
            .descendants()
            .filter_map(|c| c.try_into().ok())
            .collect();

        if !remaining_windows.is_empty() {
            rebuild_spiral_layout(&workspace, &remaining_windows)?;
            state.pending_sync.queue_containers_to_redraw(workspace.tiling_children());
        }
    }

    if !workspace.config().keep_alive
      && !workspace.has_children()
      && !workspace.is_displayed()
    {
      deactivate_workspace(workspace, state)?;
    }
  }

  Ok(())
}
