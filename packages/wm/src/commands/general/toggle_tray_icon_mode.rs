use crate::wm_state::WmState;

/// Toggles the runtime tray icon mode.
pub fn toggle_tray_icon_mode(state: &mut WmState) {
  let _ = state.toggle_tray_icon_mode();
}
