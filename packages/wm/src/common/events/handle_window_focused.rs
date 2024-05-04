use tracing::info;

use crate::{
  common::{
    commands::sync_native_focus, platform::NativeWindow, DisplayState,
  },
  containers::{commands::set_focused_descendant, Container},
  user_config::UserConfig,
  windows::traits::WindowGetters,
  wm_event::WmEvent,
  wm_state::WmState,
};

pub fn handle_window_focused(
  native_window: NativeWindow,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  if let Some(window) = found_window {
    // Ignore event if window is being hidden by the WM.
    if window.display_state() == DisplayState::Hiding {
      return Ok(());
    }

    // TODO: Log window details.
    info!("Window focused");

    // Focus is already set to the WM's focused container.
    if state.focused_container() == Some(window.clone().into()) {
      return Ok(());
    }

    // Handle overriding focus on close/minimize. After a window is closed
    // or minimized, the OS or the closed application might automatically
    // switch focus to a different window. To force focus to go to the WM's
    // target focus container, we reassign any focus events 100ms after
    // close/minimize. This will cause focus to briefly flicker to the OS
    // focus target and then to the WM's focus target.
    if state
      .unmanaged_or_minimized_timestamp
      .map(|time| time.elapsed().as_millis() < 100)
      .unwrap_or(false)
    {
      info!("Overriding native focus.");
      sync_native_focus(state)?;
      return Ok(());
    }

    // Handle focus events from windows on hidden workspaces. For example,
    // if Discord is forcefully shown by the OS when it's on a hidden
    // workspace, switch focus to Discord's workspace.
    if window.clone().display_state() == DisplayState::Hidden {
      // TODO: Log window details.
      info!("Focusing off-screen window.");
      // TODO: Focus the workspace of the hidden window.
      return Ok(());
    }

    // Update the WM's focus state.
    set_focused_descendant(window.clone().into(), None);

    state.emit_event(WmEvent::FocusChanged {
      focused_container: Into::<Container>::into(window).clone(),
    })
  }

  Ok(())
}
