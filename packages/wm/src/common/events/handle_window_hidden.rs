use tracing::info;

use crate::{
  common::{
    commands::sync_native_focus, platform::NativeWindow, DisplayState,
  },
  containers::commands::redraw,
  user_config::UserConfig,
  windows::{commands::unmanage_window, traits::WindowGetters},
  wm_state::WmState,
};

pub fn handle_window_hidden(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // TODO: Refresh monitor state.
  if native_window.is_app_bar() {
    state.app_bar_windows.retain(|w| w != &native_window);
    return Ok(());
  }

  let found_window = state.window_from_native(&native_window);

  if let Some(window) = found_window {
    // TODO: Log window details.
    info!("Window hidden");

    // Update the display state.
    if window.display_state() == DisplayState::Hiding {
      window.set_display_state(DisplayState::Hidden);
      return Ok(());
    }

    // Unmanage the window if it's not in a display state transition. Also,
    // since window events are not 100% guaranteed to be in correct order,
    // we need to ignore events where the window is not actually hidden.
    if window.display_state() == DisplayState::Shown
      && !window.native().is_visible()
    {
      unmanage_window(window, state)?;
      redraw(state, config)?;
      sync_native_focus(state)?;
    }
  }

  Ok(())
}
