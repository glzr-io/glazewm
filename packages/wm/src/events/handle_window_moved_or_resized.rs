use anyhow::Context;
use tracing::info;
use wm_common::{
  try_warn, ActiveDrag, ActiveDragOperation, FloatingStateConfig,
  FullscreenStateConfig, WindowState,
};
use wm_platform::{MouseButton, NativeWindow, Rect};

use crate::{
  commands::{
    container::{flatten_split_container, move_container_within_tree},
    window::update_window_state,
  },
  events::{
    handle_window_moved_or_resized_end,
    handle_window_moved_or_resized_start,
  },
  models::{TilingWindow, WindowContainer},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

#[allow(clippy::too_many_lines)]
pub fn handle_window_moved_or_resized(
  native_window: &NativeWindow,
  is_interactive_start: bool,
  is_interactive_end: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);

  // Update the window's state to be fullscreen or toggled from fullscreen.
  if let Some(window) = found_window {
    let is_left_click = state.dispatcher.is_mouse_down(&MouseButton::Left);

    if is_interactive_start || is_left_click {
      handle_window_moved_or_resized_start(native_window, state);
      return Ok(());
    } else if is_interactive_end
      || (!is_left_click && window.active_drag().is_some())
    {
      return handle_window_moved_or_resized_end(
        native_window,
        state,
        config,
      );
    }

    let old_frame_position = window.native_properties().frame;
    let frame_position = try_warn!(window.native().frame());

    let old_is_maximized = window.native_properties().is_maximized;
    let is_maximized = try_warn!(window.native().is_maximized());

    // Ignore duplicate location change events. Window position changes
    // can trigger multiple events (e.g. restore from maximized can trigger
    // as many as 4 identical events).
    if old_frame_position == frame_position
      && old_is_maximized == is_maximized
    {
      return Ok(());
    }

    let is_minimized = try_warn!(window.native().is_minimized());

    window.update_native_properties(|properties| {
      properties.frame = frame_position.clone();
      properties.is_maximized = is_maximized;
      properties.is_minimized = is_minimized;
    });

    // Ignore events for minimized windows. Let them be handled by the
    // handler for `PlatformEvent::WindowMinimized` instead.
    if is_minimized {
      return Ok(());
    }

    let nearest_monitor = state
      .nearest_monitor(&window.native())
      .context("Failed to get workspace of nearest monitor.")?;

    // TODO: Include this as part of the `match` statement below.
    if let Some(tiling_window) = window.as_tiling_window() {
      update_drag_state(
        tiling_window,
        &frame_position,
        &old_frame_position,
        state,
        config,
      )?;
    }

    let nearest_workspace = nearest_monitor
      .displayed_workspace()
      .context("No Workspace")?;

    let monitor_rect = if nearest_workspace.outer_gaps().is_significant() {
      nearest_monitor.native_properties().working_area
    } else {
      nearest_monitor.native_properties().bounds
    };

    let is_fullscreen = window.native().is_fullscreen(&monitor_rect)?;

    match window.state() {
      WindowState::Fullscreen(fullscreen_state) => {
        // Restore the window if it's no longer fullscreen *or* for the
        // edge case of fullscreen -> maximized -> restore from maximized.
        if (fullscreen_state.maximized || !is_fullscreen) && !is_maximized
        {
          info!("Window restored from fullscreen: {window}");

          let target_state = window
            .prev_state()
            .unwrap_or(WindowState::default_from_config(&config.value));

          update_window_state(
            window.clone(),
            target_state,
            state,
            config,
          )?;
        } else if is_maximized && !fullscreen_state.maximized {
          info!("Updating state from fullscreen -> maximized: {window}");

          update_window_state(
            window.clone(),
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: is_maximized,
              ..fullscreen_state
            }),
            state,
            config,
          )?;
        }
      }
      _ => {
        if is_maximized || is_fullscreen {
          info!("Window fullscreened: {window}");

          // Update the window to be fullscreen.
          update_window_state(
            window,
            WindowState::Fullscreen(FullscreenStateConfig {
              maximized: is_maximized,
              ..config.value.window_behavior.state_defaults.fullscreen
            }),
            state,
            config,
          )?;
        } else if matches!(window.state(), WindowState::Floating(_)) {
          // Update state with the new location of the floating window.
          info!("Updating floating window position: {window}");
          window.set_floating_placement(frame_position);
          window.set_has_custom_floating_placement(true);

          let monitor = window.monitor().context("No monitor.")?;

          // Update the window's workspace if it goes out of bounds of its
          // current workspace.
          if monitor.id() != nearest_monitor.id() {
            let updated_workspace = nearest_monitor
              .displayed_workspace()
              .context("Failed to get workspace of nearest monitor.")?;

            info!(
              "Floating window moved to new workspace: {updated_workspace}",
            );

            if let WindowContainer::NonTilingWindow(window) = &window {
              window.set_insertion_target(None);
            }

            move_container_within_tree(
              &window.into(),
              &updated_workspace.clone().into(),
              updated_workspace.child_count(),
              state,
            )?;
          }
        }
      }
    }
  }

  Ok(())
}

/// Updates the window operation based on changes in frame position.
///
/// This function determines whether a window is being moved or resized and
/// updates its operation state accordingly. If the window is being moved,
/// it's set to floating mode.
fn update_drag_state(
  window: &TilingWindow,
  frame_position: &Rect,
  old_frame_position: &Rect,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if let Some(active_drag) = window.active_drag() {
    let should_ignore = active_drag.operation.is_some()
      || frame_position == old_frame_position;

    if should_ignore {
      return Ok(());
    }

    let is_move = frame_position.height() == old_frame_position.height()
      && frame_position.width() == old_frame_position.width();

    let operation = if is_move {
      ActiveDragOperation::Moving
    } else {
      ActiveDragOperation::Resizing
    };

    window.set_active_drag(Some(ActiveDrag {
      operation: Some(operation),
      ..active_drag
    }));

    // Transition window to be floating while it's being dragged.
    if is_move {
      let parent = window.parent().context("No parent")?;

      let window = update_window_state(
        window.clone().into(),
        WindowState::Floating(FloatingStateConfig {
          centered: false,
          ..config.value.window_behavior.state_defaults.floating
        }),
        state,
        config,
      )?;

      // Windows are added for redraw on state changes, so here we need to
      // remove the window from the pending redraw.
      state
        .pending_sync
        .dequeue_container_from_redraw(window.clone());

      // Flatten the parent split container if it only contains the window.
      if let Some(split_parent) = parent.as_split() {
        if split_parent.child_count() == 1 {
          flatten_split_container(split_parent.clone())?;

          // Hacky fix to redraw siblings after flattening. The parent is
          // queued for redraw from the state change, which gets detached
          // on flatten.
          state
            .pending_sync
            .queue_containers_to_redraw(window.tiling_siblings());
        }
      }
    }
  }

  Ok(())
}
