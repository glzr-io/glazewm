use anyhow::Context;
use wm_common::{WindowState, WmEvent};

use crate::{
  containers::{
    commands::{
      detach_container, flatten_child_split_containers,
      set_focused_descendant,
    },
    traits::CommonGetters,
    WindowContainer,
  },
  windows::traits::WindowGetters,
  wm_state::WmState,
};

pub fn unmanage_window(
  window: WindowContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Create iterator of parent, grandparent, and great-grandparent.
  let ancestors = window.ancestors().take(3).collect::<Vec<_>>();

  // Get container to switch focus to after the window has been removed.
  let focus_target = state.focus_target_after_removal(&window.clone());

  detach_container(window.clone().into())?;

  // After detaching the container, flatten any redundant split containers.
  // For example, in the layout V[1 H[2]] where container 1 is detached to
  // become V[H[2]], this will then need to be flattened to V[2].
  for ancestor in ancestors.iter().rev() {
    flatten_child_split_containers(ancestor.clone())?;
  }

  state.emit_event(WmEvent::WindowUnmanaged {
    unmanaged_id: window.id(),
    unmanaged_handle: window.native().handle,
  });

  // Reassign focus to suitable target.
  if let Some(focus_target) = focus_target {
    set_focused_descendant(focus_target, None);
    state.pending_sync.focus_change = true;
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
      .pending_sync
      .containers_to_redraw
      .extend(ancestor_to_redraw.tiling_children().map(Into::into));
  }

  Ok(())
}
