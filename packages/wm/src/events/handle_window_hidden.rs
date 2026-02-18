use tracing::info;
use wm_common::{DisplayState, TilingStrategy};
use wm_platform::NativeWindow;

use crate::{
  commands::window::unmanage_window, traits::WindowGetters,
  wm_state::WmState,
};

pub fn handle_window_hidden(
  native_window: &NativeWindow,
  state: &mut WmState,
  tiling_strategy: &TilingStrategy,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);

  if let Some(window) = found_window {
    info!("Window hidden: {window}");

    // Update the display state.
    if window.display_state() == DisplayState::Hiding {
      window.set_display_state(DisplayState::Hidden);
      return Ok(());
    }

    // Unmanage the window if it's not in a display state transition. Also,
    // since window events are not 100% guaranteed to be in correct order,
    // we need to ignore events where the window is not actually hidden.
    if window.display_state() == DisplayState::Shown
      && !window.native().is_visible().unwrap_or(false)
    {
      unmanage_window(window, state, tiling_strategy)?;
    }
  }

  Ok(())
}
