use anyhow::Context;
use tracing::info;
use wm_common::{
  try_warn, ActiveDrag, ActiveDragOperation, FloatingStateConfig,
  FullscreenStateConfig, WindowState,
};
use wm_platform::{
  LengthValue, MouseButton, NativeWindow, Rect, RectDelta,
};

use crate::{
  commands::{
    container::{flatten_split_container, move_container_within_tree},
    window::update_window_state,
  },
  events::handle_window_moved_or_resized_end,
  models::{TilingWindow, WindowContainer},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

#[allow(clippy::too_many_lines)]
pub fn handle_window_moved_or_resized(
  native_window: &NativeWindow,
  // LINT: Arguments are unused for macOS, but required for Windows.
  #[allow(unused)] is_interactive_start: bool,
  #[allow(unused)] is_interactive_end: bool,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let found_window = state.window_from_native(native_window);

  if let Some(window) = found_window {
    let old_frame_position = window.native_properties().frame;
    let frame_position = try_warn!(window.native().frame());

    // Handle drag start/update/end if the WM is not paused.
    if !state.is_paused {
      // Handle windows that are being actively being dragged.
      if window.active_drag().is_some() {
        // Check if the drag operation has ended.
        let is_drag_end = {
          // On Windows, the drag operation is ended when
          // `is_interactive_end` is `true`. This corresponds to a
          // `EVENT_SYSTEM_MOVESIZEEND` event, which is unavailable on
          // macOS.
          #[cfg(target_os = "windows")]
          {
            is_interactive_end
          }
          // On macOS, the drag operation is ended when the mouse button is
          // no longer down. This is a fallback mechanism since for macOS,
          // `is_interactive_end` is always `false`. The `MouseEvent`
          // handler also catches `MouseButtonUp` events, but this provides
          // additional safety.
          // TODO: Can likely remove this check and rely 100% on the mouse
          // event handler.
          #[cfg(target_os = "macos")]
          {
            !state.dispatcher.is_mouse_down(&MouseButton::Left)
          }
        };

        if is_drag_end {
          return handle_window_moved_or_resized_end(
            &window, state, config,
          );
        }

        if let Some(tiling_window) = window.as_tiling_window() {
          update_drag_state(
            tiling_window,
            &frame_position,
            state,
            config,
          )?;
        }

        return Ok(());
      }

      // Detect whether the window is starting to be interactively moved or
      // resized by the user (e.g. via the window's drag handles).
      let is_drag_start = {
        #[cfg(target_os = "windows")]
        {
          is_interactive_start
        }
        #[cfg(target_os = "macos")]
        {
          let is_left_click =
            state.dispatcher.is_mouse_down(&MouseButton::Left);

          // Only consider the window to be being dragged if the left-click
          // is down and the cursor is within 40px margin around the
          // window's frame.
          if is_left_click {
            let frame = window.native_properties().frame.apply_delta(
              &RectDelta::new(
                LengthValue::from_px(40),
                LengthValue::from_px(40),
                LengthValue::from_px(40),
                LengthValue::from_px(40),
              ),
              None,
            );

            let cursor_position = state.dispatcher.cursor_position()?;
            frame.contains_point(&cursor_position)
          } else {
            false
          }
        }
      };

      if is_drag_start {
        window.set_active_drag(Some(ActiveDrag {
          operation: None,
          is_from_tiling: window.is_tiling_window(),
          #[cfg(target_os = "windows")]
          initial_position: old_frame_position.clone(),
          // `frame_position` is deliberately used here instead of
          // `old_frame_position` due to a quirk on macOS. When we resize
          // an AXUIElement to a value outside the allowed min/max width &
          // height, macOS doesn't actually apply that size. However, it
          // still reports the value we attempted to set until a subsequent
          // `WindowEvent::MovedOrResized` event.
          #[cfg(target_os = "macos")]
          initial_position: frame_position.clone(),
        }));

        return Ok(());
      }
    }

    let old_is_maximized = window.native_properties().is_maximized;
    let is_maximized = try_warn!(window.native().is_maximized());

    // Ignore duplicate location change events. Window position changes can
    // trigger multiple events (e.g. restore from maximized can trigger as
    // many as 4 identical events).
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
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let Some(active_drag) = window.active_drag() else {
    return Ok(());
  };

  // Ignore if the window position has not changed yet.
  if *frame_position == active_drag.initial_position {
    return Ok(());
  }

  // Determine the drag operation if not already set.
  let is_move = if let Some(operation) = active_drag.operation {
    matches!(operation, ActiveDragOperation::Move)
  } else {
    let is_move = *frame_position != active_drag.initial_position
      && frame_position.height() == active_drag.initial_position.height()
      && frame_position.width() == active_drag.initial_position.width();

    let operation = if is_move {
      ActiveDragOperation::Move
    } else {
      ActiveDragOperation::Resize
    };

    window.set_active_drag(Some(ActiveDrag {
      operation: Some(operation),
      ..active_drag.clone()
    }));

    is_move
  };

  // Transition window to be floating while it's being dragged, but only
  // after it has been moved at least 10px from its initial position. The
  // 10px threshold is to account for small movements that may be
  // accidental.
  if is_move {
    let move_distance = frame_position
      .center_point()
      .distance_between(&active_drag.initial_position.center_point());

    if move_distance >= 10.0 {
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

      // Flatten the parent split container if it only contains the window.
      // TODO: Consider doing this in `update_window_state` and
      // `move_window_in_direction` as well, so that the behavior is
      // consistent.
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
