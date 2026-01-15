use wm_common::HideMethod;

use crate::{
  models::WindowContainer, traits::WindowGetters, wm_state::WmState,
};

#[allow(clippy::needless_pass_by_value)]
pub fn set_hide_method(
  window: WindowContainer,
  hide_method: HideMethod,
  state: &mut WmState,
) -> anyhow::Result<()> {
  window.set_hide_method(Some(hide_method));

  // Queue the window to be redrawn with the new hide method.
  state.pending_sync.queue_container_to_redraw(window);

  Ok(())
}
