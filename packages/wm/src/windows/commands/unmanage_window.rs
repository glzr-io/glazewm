use anyhow::Context;

use crate::{
  containers::{
    commands::detach_container, traits::CommonGetters, WindowContainer,
  },
  windows::{traits::WindowGetters, WindowState},
  wm_event::WmEvent,
  wm_state::WmState,
};

pub fn unmanage_window(
  window: WindowContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let unmanaged_id = window.id();
  let unmanaged_handle = window.native().handle;

  let parent = window.parent().context("No parent.")?;
  let window_state = window.state();

  detach_container(window.into())?;

  state.emit_event(WmEvent::WindowUnmanaged {
    unmanaged_id,
    unmanaged_handle,
  });

  // TODO: Handle focus after removal.

  // Sibling containers need to be redrawn if the window was tiling.
  if window_state == WindowState::Tiling {
    state.add_container_to_redraw(parent.into());
  }

  Ok(())
}
