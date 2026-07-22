use tracing::info;
use wm_common::{DisplayState, HideMethod};
use wm_platform::NativeWindow;

use crate::{
  commands::window::unmanage_window, traits::WindowGetters,
  user_config::UserConfig, wm_state::WmState,
};

pub fn handle_window_hidden(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);

  if let Some(window) = found_window {
    info!("Window hidden: {window}");

    // Update the display state.
    if config.value.general.hide_method != HideMethod::PlaceInCorner
      && window.display_state() == DisplayState::Hiding
    {
      window.set_display_state(DisplayState::Hidden);
      return Ok(());
    }

    // On Windows, skip unmanagement if the window is currently cloaked.
    // Cloaking is used internally by GlazeWM (e.g. surrogate resize
    // animations) and must not be interpreted as the app hiding itself.
    // Workspace-switch cloaking (`HideMethod::Cloak`) is already handled
    // above via the `Hiding` display-state guard, so this only fires for
    // surrogate-cloaked windows whose display state is still `Shown`.
    #[cfg(target_os = "windows")]
    {
      use wm_platform::NativeWindowWindowsExt;
      if window.native().is_cloaked().unwrap_or(false) {
        return Ok(());
      }
    }

    // Unmanage the window if it's not in a display state transition. Also,
    // since window events are not 100% guaranteed to be in correct order,
    // we need to ignore events where the window is not actually hidden.
    if (config.value.general.hide_method == HideMethod::PlaceInCorner
      || window.display_state() == DisplayState::Shown)
      && !window.native().is_visible().unwrap_or(false)
    {
      unmanage_window(window, state)?;
    }
  }

  Ok(())
}
