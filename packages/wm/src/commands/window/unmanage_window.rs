use wm_common::{WindowState, WmEvent};

use crate::{
  commands::container::{
    detach_container, flatten_child_split_containers,
    set_focused_descendant,
  },
  models::WindowContainer,
  traits::{CommonGetters, WindowGetters},
  wm_state::WmState,
};

#[allow(clippy::needless_pass_by_value)]
pub fn unmanage_window(
  window: WindowContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Ensure that the window is removed from pending redraws to prevent
  // ghosting and E_INVALIDARG errors if its handle becomes invalid.
  state.pending_sync.dequeue_container_from_redraw(window.clone());

  // Create iterator of parent, grandparent, and great-grandparent.
  let ancestors = window.ancestors().take(3).collect::<Vec<_>>();

  // Get container to switch focus to after the window has been removed.
  let focus_target = state.focus_target_after_removal(&window.clone());

  detach_container(window.clone().into())?;

  // After detaching the container, flatten any redundant split containers.
  // For example, in the layout V[1 H[2]] where container 1 is detached to
  // become V[H[2]], this will then need to be flattened to V[2].
  for ancestor in ancestors.iter().rev() {
    flatten_child_split_containers(ancestor)?;
  }

  state.emit_event(WmEvent::WindowUnmanaged {
    unmanaged_id: window.id(),
    unmanaged_handle: window.native().handle,
  });

  // Reassign focus to suitable target.
  if let Some(focus_target) = focus_target {
    set_focused_descendant(&focus_target, None);
    state.pending_sync.queue_focus_change();
    state.unmanaged_or_minimized_timestamp =
      Some(std::time::Instant::now());
  }

  // Sibling containers need to be redrawn after the window is removed.
  // We redraw for all window types (tiling and floating) to ensure the
  // area previously occupied by the window is cleaned up.
  if let Some(ancestor_to_redraw) = ancestors
    .into_iter()
    .find(|ancestor| !ancestor.is_detached())
  {
    state.pending_sync.queue_containers_to_redraw(
      ancestor_to_redraw
        .descendants()
        .filter_map(|d| d.as_window_container().ok()),
    );
  }

  Ok(())
}
