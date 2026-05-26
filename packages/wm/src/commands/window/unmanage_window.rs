use anyhow::Context;
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

/// Detaches a closing window from the container tree and reflows its
/// siblings without removing the active close animation.
///
/// Called at the start of a close animation so sibling windows begin their
/// reflow animations immediately, in parallel with the close surrogate. The
/// animation state is intentionally preserved here because
/// `AnimationManager::update_internal` continues driving the surrogate and
/// sends `WM_CLOSE` once the animation completes.
#[cfg(target_os = "windows")]
pub fn detach_window_for_close(
  window: WindowContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let ancestors = window.ancestors().take(3).collect::<Vec<_>>();
  let focus_target = state.focus_target_after_removal(&window.clone());

  detach_container(window.clone().into())?;

  state.window_target_positions.remove(&window.id());
  // NOTE: `state.animation_manager.remove_animation` is intentionally
  // skipped — the close surrogate must keep running until
  // `AnimationManager::update_internal` sends `WM_CLOSE` on completion.

  for ancestor in ancestors.iter().rev() {
    flatten_child_split_containers(ancestor)?;
  }

  state.emit_event(WmEvent::WindowUnmanaged {
    unmanaged_id: window.id(),
    #[allow(clippy::cast_possible_wrap, clippy::unnecessary_cast)]
    unmanaged_handle: window.native().id().0 as isize,
  });

  if let Some(focus_target) = focus_target {
    set_focused_descendant(&focus_target, None);
    state.pending_sync.queue_focus_change();
    state.unmanaged_or_minimized_timestamp =
      Some(std::time::Instant::now());
  }

  if window.state() == WindowState::Tiling {
    let ancestor_to_redraw = ancestors
      .into_iter()
      .find(|ancestor| !ancestor.is_detached())
      .context("No ancestor to redraw.")?;

    state
      .pending_sync
      .queue_containers_to_redraw(ancestor_to_redraw.tiling_children());
  }

  Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn unmanage_window(
  window: WindowContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // Create iterator of parent, grandparent, and great-grandparent.
  let ancestors = window.ancestors().take(3).collect::<Vec<_>>();

  // Get container to switch focus to after the window has been removed.
  let focus_target = state.focus_target_after_removal(&window.clone());

  detach_container(window.clone().into())?;

  // Clean up animation tracking data.
  state.window_target_positions.remove(&window.id());
  state.animation_manager.remove_animation(&window.id());

  // After detaching the container, flatten any redundant split containers.
  // For example, in the layout V[1 H[2]] where container 1 is detached to
  // become V[H[2]], this will then need to be flattened to V[2].
  for ancestor in ancestors.iter().rev() {
    flatten_child_split_containers(ancestor)?;
  }

  state.emit_event(WmEvent::WindowUnmanaged {
    unmanaged_id: window.id(),
    #[allow(clippy::cast_possible_wrap, clippy::unnecessary_cast)]
    unmanaged_handle: window.native().id().0 as isize,
  });

  // Reassign focus to suitable target.
  if let Some(focus_target) = focus_target {
    set_focused_descendant(&focus_target, None);
    state.pending_sync.queue_focus_change();
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
      .queue_containers_to_redraw(ancestor_to_redraw.tiling_children());
  }

  Ok(())
}
