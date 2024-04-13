use crate::{
  containers::{
    commands::detach_container, traits::CommonGetters, WindowContainer,
  },
  windows::traits::WindowGetters,
  wm_event::WmEvent,
  wm_state::WmState,
};

pub fn unmanage_window(
  window: WindowContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let unmanaged_id = window.id();
  let unmanaged_handle = window.native().handle;

  detach_container(window.into())?;

  state.emit_event(WmEvent::WindowUnmanaged {
    unmanaged_id,
    unmanaged_handle,
  });

  // TODO: Handle focus after removal.

  Ok(())
}
