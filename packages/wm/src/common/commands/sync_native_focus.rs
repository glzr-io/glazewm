use anyhow::Context;

use crate::{
  common::platform::Platform, containers::Container,
  windows::traits::WindowGetters, wm_event::WmEvent, wm_state::WmState,
};

pub fn sync_native_focus(state: &mut WmState) -> anyhow::Result<()> {
  if !state.has_pending_focus_sync {
    return Ok(());
  }

  // Container that the WM believes should have focus.
  let focused_container =
    state.focused_container().context("No focused container.")?;

  let native_window = match focused_container.clone() {
    Container::TilingWindow(window) => window.native(),
    Container::NonTilingWindow(window) => window.native(),
    _ => Platform::desktop_window(),
  };

  // Set focus to the given window handle. If the container is a normal
  // window, then this will trigger `EVENT_SYSTEM_FOREGROUND` window event
  // and its handler.
  let _ = native_window.set_foreground();

  state.emit_event(WmEvent::NativeFocusSynced {
    focused_container: focused_container.clone(),
  });

  state.emit_event(WmEvent::FocusChanged {
    focused_container: focused_container,
  });

  state.has_pending_focus_sync = false;

  Ok(())
}
