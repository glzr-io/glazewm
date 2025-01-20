use anyhow::Context;
use wm_platform::{MouseMoveEvent, Platform};

use crate::{
  commands::container::set_focused_descendant, traits::CommonGetters,
  user_config::UserConfig, wm_state::WmState,
};

pub fn handle_mouse_move(
  event: &MouseMoveEvent,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // Ignore event if left/right-click is down. Otherwise, this causes focus
  // to jitter when a window is being resized by its drag handles.
  if event.is_mouse_down || !config.value.general.focus_follows_cursor {
    return Ok(());
  }

  let window_under_cursor = Platform::window_from_point(&event.point)
    .and_then(|window| Platform::root_ancestor(&window))
    .map(|root| state.window_from_native(&root))?;

  // Set focus to whichever window is currently under the cursor.
  if let Some(window) = window_under_cursor {
    let focused_container =
      state.focused_container().context("No focused container.")?;

    if focused_container.id() != window.id() {
      set_focused_descendant(&window.as_container(), None);
      state.pending_sync.mark_focus_change();
    }
  }

  Ok(())
}
