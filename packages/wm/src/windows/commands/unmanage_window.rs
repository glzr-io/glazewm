use anyhow::Context;

use crate::{
  containers::{
    commands::{detach_container, set_focused_descendant},
    traits::CommonGetters,
    WindowContainer,
  },
  windows::{traits::WindowGetters, WindowState},
  wm_event::WmEvent,
  wm_state::WmState,
};

pub fn unmanage_window(
  window: WindowContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let parent = window.parent().context("No parent.")?;
  let grandparent = parent.parent().context("No grandparent.")?;

  // Get whether the window's parent will be an empty split container.
  let has_empty_split_container =
    parent.is_split() && parent.child_count() == 1;

  // Get container to switch focus to after the window has been removed.
  let focus_target =
    state.focus_target_after_removal(&window.clone().into());

  detach_container(window.clone().into())?;

  state.emit_event(WmEvent::WindowUnmanaged {
    unmanaged_id: window.id(),
    unmanaged_handle: window.native().handle,
  });

  // Reassign focus to suitable target.
  if let Some(focus_target) = focus_target {
    set_focused_descendant(focus_target, None);
    state.has_pending_focus_sync = true;
    state.unmanaged_or_minimized_timestamp =
      Some(std::time::Instant::now());
  }

  // Sibling containers need to be redrawn if the window was tiling.
  if window.state() == WindowState::Tiling {
    state.add_container_to_redraw(match has_empty_split_container {
      true => grandparent.into(),
      false => parent.into(),
    });
  }

  Ok(())
}
