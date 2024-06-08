use anyhow::Context;

use crate::{
  common::platform::{MouseMoveEvent, Platform},
  containers::{commands::set_focused_descendant, traits::CommonGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_mouse_move(
  mouse_move_event: MouseMoveEvent,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if !config.value.general.focus_follows_cursor {
    return Ok(());
  }

  let window_under_cursor =
    Platform::window_from_point(&mouse_move_event.point)
      .and_then(|window| Platform::root_ancestor(&window))
      .map(|root| state.window_from_native(&root))?;

  // Set focus to whichever window is currently under the cursor.
  if let Some(window) = window_under_cursor {
    let focused_container =
      state.focused_container().context("No focused container.")?;

    if focused_container.id() != window.id() {
      set_focused_descendant(window.as_container(), None);
      state.has_pending_focus_sync = true;
    }
  }

  Ok(())
}
