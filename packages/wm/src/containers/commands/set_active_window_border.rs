use crate::common::platform::NativeWindow;
use crate::user_config::UserConfig;
use crate::wm_state::WmState;
use anyhow::{Error, Result};
use std::convert::TryInto;
use std::os::raw::c_void;
use windows::Win32::Graphics::Dwm::{
  DwmSetWindowAttribute, DWMWA_BORDER_COLOR,
};

pub fn handle_set_active_window_border(
  state: &mut WmState,
  target: NativeWindow,
  config: &UserConfig,
) -> Result<(), Error> {
  if let Some(last_focused) = &state.active_border_window {
    // Clear old window border
    let inactive_color = if config.value.focus_borders.inactive.enabled {
      rgb_to_uint(config.value.focus_borders.inactive.color.clone())
    } else {
      0xFFFFFFFF
    };
    unsafe {
      let inactive_color: u32 = 0xFFFFFFFF; // replace with your color
      let pv_attribute: *const c_void =
        &inactive_color as *const _ as *const c_void;
      DwmSetWindowAttribute(
        last_focused.handle,
        DWMWA_BORDER_COLOR,
        pv_attribute,
        std::mem::size_of::<u32>() as u32,
      )?;
    }
  }

  // Set new window border
  let active_color = if config.value.focus_borders.active.enabled {
    rgb_to_uint(config.value.focus_borders.active.color.clone())
  } else {
    0xFFFFFFFF
  };
  unsafe {
    let active_color: u32 =
      rgb_to_uint(config.value.focus_borders.active.color.clone()); // replace with your color
    let pv_attribute: *const c_void =
      &active_color as *const _ as *const c_void;
    DwmSetWindowAttribute(
      target.handle,
      DWMWA_BORDER_COLOR,
      pv_attribute,
      std::mem::size_of::<u32>() as u32,
    )?;
  }

  state.active_border_window = Some(target);

  Ok(())
}

fn rgb_to_uint(rgb: String) -> u32 {
  let c = rgb.chars().collect::<Vec<char>>();
  let bgr = format!("{}{}{}{}{}{}", c[5], c[6], c[3], c[4], c[1], c[2]);
  u32::from_str_radix(&bgr, 16).unwrap()
}
