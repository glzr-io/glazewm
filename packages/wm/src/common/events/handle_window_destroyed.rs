use tracing::info;

use crate::{
  common::platform::NativeWindow, windows::commands::unmanage_window, 
  wm_state::WmState,
  containers::traits::CommonGetters,
  workspaces::commands::deactivate_workspace
};

pub fn handle_window_destroyed(
  native_window: NativeWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  // Unmanage the window if it's currently managed.
  if let Some(window) = found_window {
    // TODO: Log window details.
    let bind_workspace = window.workspace();

    info!("Window closed");
    unmanage_window(window, state)?;

    if let Some(workspace) = bind_workspace {
      if !workspace.config().keep_alive && !workspace.has_children() && !workspace.is_displayed() {
        deactivate_workspace(workspace, state)?;
      }
    }
  }

  Ok(())
}
