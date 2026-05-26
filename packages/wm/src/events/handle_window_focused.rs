use anyhow::Context;
use tracing::info;
use wm_common::{DisplayState, FocusTrigger, WindowRuleEvent, WmEvent};
use wm_platform::NativeWindow;

use crate::{
  commands::{
    container::set_focused_descendant, window::run_window_rules,
    workspace::focus_workspace,
  },
  models::WorkspaceTarget,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_window_focused(
  native_window: &NativeWindow,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);
  let focused_container =
    state.focused_container().context("No focused container.")?;

  // Update the focus sync state. If the OS focused window is not same as
  // the WM's focused container, then the focus is not synced.
  state.is_focus_synced = match focused_container.as_window_container() {
    Ok(window) => *window.native() == *native_window,
    _ => native_window.is_desktop_window().unwrap_or(false),
  };

  // Handle overriding focus on close/minimize. After a window is closed
  // or minimized, the OS or the closed application might automatically
  // switch focus to a different window. To force focus to go to the WM's
  // target focus container, we reassign any focus events 100ms after
  // close/minimize. This will cause focus to briefly flicker to the OS
  // focus target and then to the WM's focus target.
  if should_override_focus(state) {
    state.pending_sync.queue_focus_change();
    return Ok(());
  }

  // Ignore the focus event if window is being hidden by the WM.
  if let Some(window) = &found_window {
    if window.display_state() == DisplayState::Hiding {
      return Ok(());
    }
  }

  // Focus effect should be updated for any change in focus that shouldn't
  // be overwritten. The incoming focus event at this point is either:
  //  1. WM's focus container (window or workspace). This is the desktop
  //     window in the case of a workspace.
  //  2. An ignored window.
  //  3. A window that received manual focus.
  state.pending_sync.queue_focused_effect_update();

  if let Some(window) = found_window {
    let workspace = window.workspace().context("No workspace")?;

    // Native focus has been synced to the WM's focused container.
    if focused_container == window.clone().into() {
      state.is_focus_synced = true;
      state.pending_sync.queue_workspace_to_reorder(workspace);
      // Keyboard/WM-command focus: always eligible for focus animation.
      #[cfg(target_os = "windows")]
      if config.value.animations.focus_change.enabled {
        state.pending_sync.queue_focus_animation(window.id());
        state.pending_sync.queue_container_to_redraw(window.clone());
      }
      return Ok(());
    }

    info!("Window manually focused: {window}");

    // Handle focus events from windows on hidden workspaces. For example,
    // if Discord is forcefully shown by the OS when it's on a hidden
    // workspace, switch focus to Discord's workspace.
    if window.display_state() == DisplayState::Hidden
      && !workspace.is_displayed()
    {
      info!("Focusing off-screen window: {window}");

      focus_workspace(
        WorkspaceTarget::Name(workspace.config().name),
        state,
        config,
      )?;
    }

    // Update the WM's focus state.
    set_focused_descendant(&window.clone().into(), None);

    // Run window rules for focus events.
    run_window_rules(
      window.clone(),
      &WindowRuleEvent::Focus,
      state,
      config,
    )?;

    state.is_focus_synced = true;
    state.pending_sync.queue_workspace_to_reorder(workspace);

    // Manual (mouse-click) focus: only animate when trigger is `All`.
    #[cfg(target_os = "windows")]
    {
      let fc = &config.value.animations.focus_change;
      if fc.enabled && fc.trigger == FocusTrigger::All {
        state.pending_sync.queue_focus_animation(window.id());
        state.pending_sync.queue_container_to_redraw(window.clone());
      }
    }

    // Broadcast the focus change event.
    state.emit_event(WmEvent::FocusChanged {
      focused_container: window.to_dto()?,
    });
  }

  Ok(())
}

/// Returns true if focus should be reassigned to the WM's focus container.
fn should_override_focus(state: &WmState) -> bool {
  let has_recent_unmanage = state
    .unmanaged_or_minimized_timestamp
    .is_some_and(|time| time.elapsed().as_millis() < 100);

  has_recent_unmanage && !state.is_focus_synced
}
