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
  // Also ignore if the OS focused window isn't the same as the WM's
  // focused window.
  if event.is_mouse_down
    || !state.is_focus_synced
    || !config.value.general.focus_follows_cursor
  {
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
      state.pending_sync.queue_focus_change();
    }
  } else {
    // Focus the monitor if no window is under the cursor.
    let cursor_monitor = state
      .monitor_at_point(&event.point)
      .context("No monitor under cursor.")?;

    let focused_monitor = state
      .focused_container()
      .context("No focused container.")?
      .monitor()
      .context("Focused container has no monitor.")?;

    // Avoid setting focus to the same monitor.
    if cursor_monitor.id() != focused_monitor.id() {
      set_focused_descendant(&cursor_monitor.as_container(), None);
      state.pending_sync.queue_focus_change();
    }
  }

  Ok(())
}
