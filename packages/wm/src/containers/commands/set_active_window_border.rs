use crate::common::platform::NativeWindow;
use crate::user_config::UserConfig;
use crate::wm_state::WmState;
use anyhow::{Error, Result};
use std::convert::TryInto;
use windows::Win32::Graphics::Dwm::{
  DWMWA_COLOR_NONE,
};

pub fn handle_set_active_window_border(
  state: &mut WmState,
  target: NativeWindow,
  config: &UserConfig,
) -> Result<(), Error> {
  // Clear old window border
  let inactive_border_color = if config.value.focus_borders.inactive.enabled {
    DWMWA_COLOR_NONE
  } else {
    config.value.focus_borders.inactive.color.to_bgr().clone()
  };
  target.set_border_color(inactive_border_color)?;

  // Set new window border
  target.set_border_color(
    config.value.focus_borders.active.color.to_bgr().clone(),
  )?;

  state.active_border_window = Some(target);

  Ok(())
}
