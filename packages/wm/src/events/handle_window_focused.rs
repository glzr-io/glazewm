use anyhow::Context;
use tracing::info;
use wm_common::{DisplayState, WindowRuleEvent};
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

  if let Some(window) = found_window {
    // Ignore the focus event if:
    // 1. Window is being hidden by the WM.
    // 2. Focus is already set to the WM's focused container.
    if window.display_state() == DisplayState::Hiding
      || state.focused_container() == Some(window.clone().into())
    {
      return Ok(());
    }

    info!("Window focused: {window}");

    // Handle overriding focus on close/minimize. After a window is closed
    // or minimized, the OS or the closed application might automatically
    // switch focus to a different window. To force focus to go to the WM's
    // target focus container, we reassign any focus events 100ms after
    // close/minimize. This will cause focus to briefly flicker to the OS
    // focus target and then to the WM's focus target.
    if state
      .unmanaged_or_minimized_timestamp
      .is_some_and(|time| time.elapsed().as_millis() < 100)
    {
      info!("Overriding native focus.");
      state.pending_sync.focus_change = true;
      return Ok(());
    }

    // Handle focus events from windows on hidden workspaces. For example,
    // if Discord is forcefully shown by the OS when it's on a hidden
    // workspace, switch focus to Discord's workspace.
    if window.clone().display_state() == DisplayState::Hidden {
      info!("Focusing off-screen window: {window}");

      let workspace = window.workspace().context("No workspace")?;
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

    state.pending_sync.update_focused_window_effect = true;
  }

  Ok(())
}
