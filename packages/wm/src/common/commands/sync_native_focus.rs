use anyhow::Context;

use crate::{
  common::platform::Platform,
  containers::{traits::CommonGetters, Container},
  windows::traits::WindowGetters,
  wm_event::WmEvent,
  wm_state::WmState,
};

pub fn sync_native_focus(state: &mut WmState) -> anyhow::Result<()> {
  if !state.has_pending_focus_sync {
    return Ok(());
  }

  // Get the container that the WM believes should have focus.
  let focused_container =
    state.focused_container().context("No focused container.")?;

  let native_window = match focused_container.clone() {
    Container::TilingWindow(window) => window.native(),
    Container::NonTilingWindow(window) => window.native(),
    _ => Platform::desktop_window(),
  };

  // Set focus to the given window handle. If the container is a normal
  // window, then this will trigger a `PlatformEvent::WindowFocused` event.
  if Platform::foreground_window() != native_window {
    let _ = native_window.set_foreground();
  }

  // TODO: Change z-index of workspace windows that match the focused
  // container's state. Make sure not to decrease z-index for floating
  // windows that are always on top.

  state.emit_event(WmEvent::FocusChanged {
    focused_container: focused_container.to_dto()?,
  });

  state.has_pending_focus_sync = false;

  Ok(())
}
