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

  let root_window = Platform::get_root_ancestor(&window_under_cursor)?;

  // prevent spam focusing the same window 
  // by checking the window under cursor against the WM's currently focused window
  if let Some(window) = state.window_from_native(&root_window) {
    if (state.focused_container() != Some(window.as_container())) {
      set_focused_descendant(window.as_container(), None);
      state.has_pending_focus_sync = true;
      redraw(state)?;
      sync_native_focus(state)?;
    }
  }
  Ok(())
}
