use tracing::info;
use wm_common::{DisplayState, HideMethod};
use wm_platform::NativeWindow;

use crate::{
  commands::window::{park_window, unmanage_window},
  traits::WindowGetters,
  user_config::UserConfig,
  wm_state::WmState,
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

    // Unmanage the window if it's not in a display state transition. Also,
    // since window events are not 100% guaranteed to be in correct order,
    // we need to ignore events where the window is not actually hidden.
    if (config.value.general.hide_method == HideMethod::PlaceInCorner
      || window.display_state() == DisplayState::Shown)
      && !window.native().is_visible().unwrap_or(false)
    {
      // Windows get cloaked by the shell when switching to a different
      // virtual desktop. Park these windows instead of unmanaging them, so
      // that they can be restored to their previous position when
      // switching back.
      //
      // The shell cloak is checked in addition to the window's virtual
      // desktop, since a window can get cloaked before the current virtual
      // desktop is updated.
      #[cfg(target_os = "windows")]
      {
        use wm_platform::NativeWindowWindowsExt;

        let is_moved_off_desktop =
          window.native().is_cloaked_by_shell().unwrap_or(false)
            || !window
              .native()
              .is_on_current_virtual_desktop()
              .unwrap_or(true);

        if is_moved_off_desktop {
          return park_window(window, state);
        }
      }

      unmanage_window(window, state)?;
    }
  }

  Ok(())
}
