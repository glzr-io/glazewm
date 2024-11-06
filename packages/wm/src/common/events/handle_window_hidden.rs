use tracing::info;

use crate::{
  common::{platform::NativeWindow, DisplayState},
  windows::{commands::unmanage_window, traits::WindowGetters},
  wm_state::WmState,
};

pub fn handle_window_hidden(
  native_window: NativeWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  if let Some(window) = found_window {
    // TODO: Log window details.
    info!("Window hidden");

    // Update the display state.
    if window.display_state() == DisplayState::Hiding {
      window.set_display_state(DisplayState::Hidden);
      return Ok(());
    }

    // Unmanage thed window if it's not in a display state transition. Also,
    // since window events are not 100% guaranteed to be in correct order,
    // we need to ignore events where the window is not actually hidden.
    if window.display_state() == DisplayState::Shown
      && !window.native().is_visible().unwrap_or(false)
    {
      unmanage_window(window, state)?;
    }
  }

  Ok(())
}
