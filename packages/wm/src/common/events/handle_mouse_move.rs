use crate::common::commands::sync_native_focus;
use crate::common::platform::{MouseMoveEvent, Platform};
use crate::containers::commands::{redraw, set_focused_descendant};
use crate::containers::traits::CommonGetters;
use crate::user_config::UserConfig;
use crate::wm_state::WmState;

pub fn handle_mouse_move(
  mouse_move_event: MouseMoveEvent,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if !config.value.general.focus_follows_cursor {
    return Ok(());
  }
  
  let mut window_under_cursor =
    Platform::window_from_point(&mouse_move_event.point).unwrap();

  // walk the window tree to find the top-level window
  while let Ok(parent) = Platform::get_parent(&window_under_cursor) {
    window_under_cursor = parent;
  }

  // prevent spam focusing the same window by checking the target window against the currently focused window
  if let Some(window) = state.window_from_native(&window_under_cursor) {
    let currently_focused_container = state.focused_container().unwrap();
    let tiling_window =
      currently_focused_container.as_tiling_window().unwrap();
    if (tiling_window.id() != window.id()) {
      set_focused_descendant(window.as_container(), None);
      state.has_pending_focus_sync = true;
      redraw(state)?;
      sync_native_focus(state)?;
    }
  }
  Ok(())
}
