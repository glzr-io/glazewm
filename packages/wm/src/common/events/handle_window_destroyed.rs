use tracing::info;

use crate::{
  common::platform::NativeWindow, windows::commands::unmanage_window,
  wm_state::WmState,
};

pub fn handle_window_destroyed(
  native_window: NativeWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // TODO: Refresh monitor state.
  if native_window.is_app_bar() {
    state.app_bar_windows.retain(|w| w != &native_window);
    return Ok(());
  }

  let found_window = state.window_from_native(&native_window);

  // Unmanage the window if it's currently managed.
  if let Some(window) = found_window {
    // TODO: Log window details.
    info!("Window closed");
    unmanage_window(window, state)?;
  }

  Ok(())
}
