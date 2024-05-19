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
  // Create iterator of window's parent, grandparent, and great-
  // grandparent.
  let ancestors = window.ancestors().take(3).collect::<Vec<_>>();

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
    let ancestor_to_redraw = ancestors
      .into_iter()
      .find(|ancestor| !ancestor.is_detached())
      .context("No ancestor to redraw.")?;

    state
      .containers_to_redraw
      .extend(ancestor_to_redraw.tiling_children().map(Into::into));
  }

  Ok(())
}
