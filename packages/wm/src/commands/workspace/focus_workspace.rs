use anyhow::Context;
use tracing::info;

use wm_common::{WindowState, WorkspaceSwitchIrisOrigin};

use super::activate_workspace;
use crate::{
  commands::{
    container::set_focused_descendant, workspace::deactivate_workspace,
  },
  models::{Container, WorkspaceTarget},
  pending_sync::IrisSwitchRequest,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Focuses a workspace by a given target.
///
/// This target can be a workspace name, the most recently focused
/// workspace, the next workspace, the previous workspace, or the workspace
/// in a given direction from the currently focused workspace.
///
/// The workspace will be activated if it isn't already active.
pub fn focus_workspace(
  target: WorkspaceTarget,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_workspace = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .context("No workspace is currently focused.")?;

  let (target_workspace_name, target_workspace) =
    state.workspace_by_target(&focused_workspace, target, config)?;

  // Retrieve or activate the target workspace by its name.
  let target_workspace = match target_workspace {
    Some(_) => anyhow::Ok(target_workspace),
    _ => match target_workspace_name {
      Some(name) => {
        activate_workspace(Some(&name), None, state, config)?;

        Ok(state.workspace_by_name(&name))
      }
      _ => Ok(None),
    },
  }?;

  if let Some(target_workspace) = target_workspace {
    info!("Focusing workspace: {target_workspace}");

    // Get the currently displayed workspace on the same monitor that the
    // workspace to focus is on.
    let displayed_workspace = target_workspace
      .monitor()
      .and_then(|monitor| monitor.displayed_workspace())
      .context("No workspace is currently displayed.")?;

    // Set focus to whichever window last had focus in workspace. If the
    // workspace has no windows, then set focus to the workspace itself.
    let container_to_focus = target_workspace
      .descendant_focus_order()
      .next()
      .unwrap_or_else(|| target_workspace.clone().into());

    set_focused_descendant(&container_to_focus, None);
    state.pending_sync.queue_focus_change();

    // Only set up the workspace-switch slide animation when the target
    // workspace differs from what is currently displayed on its monitor.
    // For cross-monitor focus switches (e.g. focusing workspace 8 which is
    // already displayed on monitor 2), `target == displayed`, so no
    // workspace layout change occurs on the target monitor and no slide
    // should play.
    if target_workspace.id() != displayed_workspace.id() {
      let ws_anim = &config.value.animations.workspace_switch;

      if ws_anim.enabled && ws_anim.style.is_iris() {
        // Iris wipe: request a snapshot-overlay wipe and let the real windows
        // switch instantly underneath. No per-window incoming/outgoing marking
        // is done — the overlay (not surrogates) drives all the visuals.
        if let Some(monitor) = target_workspace.monitor() {
          let props = monitor.native_properties();
          let bounds = &props.bounds;
          let center = (
            bounds.x() + bounds.width() / 2,
            bounds.y() + bounds.height() / 2,
          );
          let (origin_x, origin_y) =
            iris_origin_point(&ws_anim.iris_origin, bounds, center, &container_to_focus);
          state.pending_sync.request_iris_switch(IrisSwitchRequest {
            monitor_x: bounds.x(),
            monitor_y: bounds.y(),
            monitor_width: bounds.width(),
            monitor_height: bounds.height(),
            monitor_handle: props.handle,
            origin_x,
            origin_y,
          });
        }
      } else {
        // Derive slide direction before queuing redraws so `platform_sync`
        // can build per-window surrogates with the correct direction.
        let direction = workspace_switch_direction(
          &target_workspace.config().name,
          &focused_workspace.config().name,
          config,
        );
        state.pending_sync.set_workspace_switch_direction(direction);

        // Mark windows on the incoming workspace to slide in. Minimized
        // windows are excluded — they have no visible content to animate and
        // including them causes flicker when the animation system tries to
        // snapshot them.
        for window in target_workspace
          .descendants()
          .filter_map(|c| c.as_window_container().ok())
          .filter(|w| w.state() != WindowState::Minimized)
        {
          state
            .pending_sync
            .setup_workspace_switch_incoming(window.id());
        }

        // Cancel in-flight animations for outgoing windows and mark them for
        // the outgoing surrogate slide-out. Minimized windows are excluded
        // for the same reason as above.
        for window in displayed_workspace
          .descendants()
          .filter_map(|c| c.as_window_container().ok())
          .filter(|w| w.state() != WindowState::Minimized)
        {
          state.animation_manager.remove_animation(&window.id());
          state
            .pending_sync
            .setup_workspace_switch_outgoing(window.id());
        }
      }
    }

    // Display the workspace to switch focus to.
    state
      .pending_sync
      .queue_container_to_redraw(displayed_workspace)
      .queue_container_to_redraw(target_workspace);

    // Get empty workspace to destroy (if one is found). Cannot destroy
    // empty workspaces if they're the only workspace on the monitor.
    let workspace_to_destroy =
      state.workspaces().into_iter().find(|workspace| {
        !workspace.config().keep_alive
          && !workspace.has_children()
          && !workspace.is_displayed()
      });

    if let Some(workspace) = workspace_to_destroy {
      deactivate_workspace(workspace, state)?;
    }

    // Save the currently focused workspace as recent.
    state.recent_workspace_name = Some(focused_workspace.config().name);
    state.pending_sync.queue_cursor_jump();
  }

  Ok(())
}

/// Returns `+1` if `target_name` has a higher config index than
/// `focused_name`, `-1` if lower, or `0` if undetermined.
///
/// Used to derive the slide direction for the workspace-switch animation.
fn workspace_switch_direction(
  target_name: &str,
  focused_name: &str,
  config: &UserConfig,
) -> i32 {
  let names: Vec<&str> = config
    .value
    .workspaces
    .iter()
    .map(|w| w.name.as_str())
    .collect();

  let target_idx = names.iter().position(|n| *n == target_name);
  let focused_idx = names.iter().position(|n| *n == focused_name);

  match (focused_idx, target_idx) {
    (Some(fi), Some(ti)) if ti > fi => 1,
    (Some(fi), Some(ti)) if ti < fi => -1,
    _ => 0,
  }
}

/// Computes the iris-wipe circle origin (screen pixels) for the configured
/// mode.
///
/// Falls back to `center` (the monitor center) when the cursor is on another
/// monitor or the focused container is not a positioned window.
fn iris_origin_point(
  origin: &WorkspaceSwitchIrisOrigin,
  monitor_bounds: &wm_platform::Rect,
  center: (i32, i32),
  focused: &Container,
) -> (i32, i32) {
  match origin {
    WorkspaceSwitchIrisOrigin::Center => center,
    WorkspaceSwitchIrisOrigin::Cursor => wm_platform::cursor_position()
      .filter(|&(x, y)| {
        x >= monitor_bounds.x()
          && x < monitor_bounds.x() + monitor_bounds.width()
          && y >= monitor_bounds.y()
          && y < monitor_bounds.y() + monitor_bounds.height()
      })
      .unwrap_or(center),
    WorkspaceSwitchIrisOrigin::FocusedWindow => focused
      .as_window_container()
      .ok()
      .and_then(|w| w.to_rect().ok())
      .map(|r| (r.x() + r.width() / 2, r.y() + r.height() / 2))
      .unwrap_or(center),
  }
}
