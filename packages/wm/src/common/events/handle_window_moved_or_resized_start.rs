use crate::{
  common::platform::NativeWindow,
  containers::WindowContainer,
  windows::{
    traits::WindowGetters,
    active_drag::{ActiveDragOperation},
  },
  wm_state::WmState,
};

/// Handles the event for when a window is started being moved or resized
/// by the user (e.g. via the window's drag handles).
pub fn handle_window_moved_or_resized_start(
  native_window: NativeWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(&native_window);

  if let Some(WindowContainer::TilingWindow(moved_window)) = found_window {
    let mut active_drag = moved_window.active_drag();
    active_drag.operation = Some(ActiveDragOperation::Waiting);
    moved_window.set_active_drag(active_drag);
  }

  Ok(())
}
