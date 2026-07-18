use anyhow::Context;
#[cfg(target_os = "windows")]
use wm_platform::NativeWindowWindowsExt;
use tracing::warn;
use wm_common::{WindowState, WmEvent};
use wm_platform::NativeWindow;

use crate::{
  commands::container::{
    detach_container, flatten_child_split_containers,
    set_focused_descendant,
  },
  models::{Monitor, WindowContainer},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Resizes the native window to the monitor workspace area (outer gaps and
/// working area), matching a single tiled window, before the WM detaches.
///
/// Best-effort: logs and continues on failure so unmanage still runs.
pub(crate) fn snap_native_window_to_external_monitor_workspace(
  window: &WindowContainer,
  monitor: &Monitor,
  config: &UserConfig,
) {
  if let Err(err) =
    try_snap_native_window_to_external_monitor_workspace(
      window, monitor, config,
    )
  {
    warn!(
      ?err,
      "Failed to snap window to workspace extent before unmanage"
    );
  }
}

fn try_snap_native_window_to_external_monitor_workspace(
  window: &WindowContainer,
  monitor: &Monitor,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let tiling_rect =
    monitor.max_workspace_rect_from_gaps(&config.value.gaps);
  let frame =
    tiling_rect.apply_delta(&window.total_border_delta()?, None);

  #[cfg(target_os = "windows")]
  {
    let should_restore = window.native().is_minimized()?
      || window.native().is_maximized()?;

    if should_restore {
      window.native().restore(Some(&frame))?;
    }
  }

  window.native().set_frame(&frame)?;

  window.update_native_properties(|properties| {
    properties.frame = frame;
  });

  Ok(())
}

/// Resizes a not-yet-managed native window to the monitor workspace area
/// (outer gaps and working area), matching a single tiled window.
///
/// Used when a window opens directly on a monitor without a WM workspace
/// (e.g. when `multi_monitor_workspaces` is disabled), so freshly opened
/// windows fill the monitor consistently with the move/unmanage path. The
/// window stays OS-managed afterwards.
///
/// Mirrors `snap_native_window_to_external_monitor_workspace` for windows
/// that have no `WindowContainer` yet.
///
/// Best-effort: logs and continues on failure.
pub(crate) fn snap_new_native_window_to_external_monitor_workspace(
  native_window: &NativeWindow,
  monitor: &Monitor,
  config: &UserConfig,
) {
  if let Err(err) =
    try_snap_new_native_window_to_external_monitor_workspace(
      native_window,
      monitor,
      config,
    )
  {
    warn!(
      ?err,
      "Failed to snap new window to workspace extent before remanage."
    );
  }
}

fn try_snap_new_native_window_to_external_monitor_workspace(
  native_window: &NativeWindow,
  monitor: &Monitor,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let tiling_rect =
    monitor.max_workspace_rect_from_gaps(&config.value.gaps);

  // A new window has no adjust border delta yet, so only the shadow
  // borders are compensated (matching `total_border_delta` for a window
  // with `RectDelta::zero()` borders). Without this, the OS places the
  // window's shadow-inclusive frame inside the tiling rect, leaving the
  // visible window slightly smaller than the workspace extent.
  #[cfg(target_os = "windows")]
  let frame =
    tiling_rect.apply_delta(&native_window.shadow_borders()?, None);
  #[cfg(not(target_os = "windows"))]
  let frame = tiling_rect;

  #[cfg(target_os = "windows")]
  {
    if native_window.is_minimized()? || native_window.is_maximized()? {
      native_window.restore(Some(&frame))?;
    }
  }

  native_window.set_frame(&frame)?;

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
