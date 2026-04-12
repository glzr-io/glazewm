use wm_common::TrayIconMode;

use crate::wm_state::WmState;

/// Sets the runtime tray icon mode.
pub fn set_tray_icon_mode(mode: TrayIconMode, state: &mut WmState) {
  let _ = state.set_tray_icon_mode(mode);
}
