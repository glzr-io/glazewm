use anyhow::Context;
use wm_common::try_warn;
use wm_platform::{MouseButton, MouseEvent};

use crate::{
  commands::container::set_focused_descendant,
  events::handle_window_moved_or_resized_end,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_mouse_move(
  event: &MouseEvent,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // Ignore mouse move events if the WM is paused. The mouse listener
  // should anyways be disabled when the WM is paused, but this is just in
  // case any events slipped through while disabling.
  if state.is_paused {
    return Ok(());
  }

  if let MouseEvent::MouseButtonUp { button, .. } = event {
    if *button == MouseButton::Left {
      // On macOS, this is the main way to detect when a window drag
      // operation has ended, since `is_interactive_end` is always false on
      // `WindowEvent::MovedOrResized`.
      let active_drag_windows = state
        .windows()
        .into_iter()
        .filter(|window| window.active_drag().is_some());

      // Only one window should ever be actively dragged at a time, but
      // just in case, iterate over all active drag windows.
      for window in active_drag_windows {
        let new_rect = try_warn!(window.native().frame());

        window.update_native_properties(|properties| {
          properties.frame = new_rect;
        });

        handle_window_moved_or_resized_end(&window, state, config)?;
      }
    }

    return Ok(());
  }

  if let MouseEvent::MouseMove {
    pressed_buttons,
    notification,
    position,
    ..
  } = event
  {
    // Ignore event if left/right-click is down. Otherwise, this causes
    // focus to jitter when a window is being resized by its drag
    // handles. Also ignore if the OS focused window isn't the same as
    // the WM's focused window.
    if pressed_buttons.contains(&MouseButton::Left)
      || pressed_buttons.contains(&MouseButton::Right)
      || !state.is_focus_synced
      || !config.value.general.focus_follows_cursor
    {
      return Ok(());
    }

    let window_under_cursor = {
      #[cfg(target_os = "macos")]
      {
        notification.0.below_window_id().and_then(|window_id| {
          use crate::traits::WindowGetters;

          state
            .windows()
            .into_iter()
            .find(|w| w.native().id() == window_id)
        })
      }
      #[cfg(target_os = "windows")]
      {
        state
          .dispatcher
          .window_from_point(&event.point)?
          .and_then(|native| state.window_from_native(&native))
      }
    };

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
        .monitor_at_point(position)
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
  }

  Ok(())
}
