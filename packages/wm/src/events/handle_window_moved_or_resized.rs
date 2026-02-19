use anyhow::Context;
use wm_common::{
  try_warn, ActiveDrag, ActiveDragOperation, DisplayState,
  FloatingStateConfig, FullscreenStateConfig, HideMethod, WindowState,
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
  models::{Monitor, NonTilingWindow, WindowContainer},
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

    window.update_native_properties(|properties| {
      properties.frame = frame_position.clone();
    });

    // Handle windows that are actively being dragged.
    if !state.is_paused && window.active_drag().is_some() {
      let is_drag_end = {
        // On Windows, the drag operation has ended when
        // `is_interactive_end` is `true`. This corresponds to a
        // `EVENT_SYSTEM_MOVESIZEEND` event, which is unavailable on macOS.
        #[cfg(target_os = "windows")]
        {
          is_interactive_end
        }
        // On macOS, the drag operation has ended when the mouse button is
        // no longer down. This is a fallback mechanism since for macOS,
        // `is_interactive_end` is always `false`. The `MouseEvent` handler
        // also catches `MouseButtonUp` events, but this provides
        // additional safety.
        // TODO: Can probably remove this check and rely 100% on the mouse
        // event handler.
        #[cfg(target_os = "macos")]
        {
          !state.dispatcher.is_mouse_down(&MouseButton::Left)
        }
      };

      if is_drag_end {
        return handle_window_moved_or_resized_end(&window, state, config);
      }

      return update_drag_state(&window, &frame_position, state, config);
    }

    let old_is_maximized = window.native_properties().is_maximized;
    let is_maximized = try_warn!(window.native().is_maximized());

    // Ignore duplicate move/resize events. Window position changes can
    // trigger multiple events. For example, restoring from maximized can
    // trigger as many as 4 identical events on Windows.
    if old_frame_position == frame_position
      && old_is_maximized == is_maximized
      && !is_interactive_start
    {
      return Ok(());
    }

    window.update_native_properties(|properties| {
      properties.is_maximized = is_maximized;
    });

    let is_minimized = try_warn!(window.native().is_minimized());

    // Ignore events for minimized windows. Let them be handled by the
    // `PlatformEvent::WindowMinimized` event handler instead.
    if is_minimized {
      return Ok(());
    }

    // Detect whether the window is starting to be interactively moved or
    // resized by the user (e.g. via the window's drag handles).
    let is_drag_start = !state.is_paused && {
      #[cfg(target_os = "windows")]
      {
        // Drag events can be valid for all window states apart from
        // minimized.
        is_interactive_start
          && !matches!(window.state(), WindowState::Minimized)
      }
      #[cfg(target_os = "macos")]
      {
        // Drag events are never valid for minimized or maximized windows.
        let is_valid_state = !matches!(
          window.state(),
          WindowState::Fullscreen(FullscreenStateConfig {
            maximized: true,
            ..
          }) | WindowState::Minimized
        );

        let is_dragging_other_window =
          state.windows().iter().any(|w| w.active_drag().is_some());

        let is_left_click =
          state.dispatcher.is_mouse_down(&MouseButton::Left);

        // Only consider the window to be dragging if:
        //  1. The window is not minimized or maximized.
        //  2. No other window is being dragged.
        //  3. Left-click is down.
        //  4. The cursor is within 40px margin around the window's frame.
        if is_valid_state && !is_dragging_other_window && is_left_click {
          // The window frame can lag behind the cursor when moving or
          // resizing quickly, so allow for a bit of leeway.
          let frame_to_check = frame_position.apply_delta(
            &RectDelta::new(
              LengthValue::from_px(40),
              LengthValue::from_px(40),
              LengthValue::from_px(40),
              LengthValue::from_px(40),
            ),
            None,
          );

          // TODO: Might be more robust to also check if the window under
          // the cursor (i.e. via `dispatcher.window_from_point`) is not a
          // different window.
          let cursor_position = state.dispatcher.cursor_position()?;
          frame_to_check.contains_point(&cursor_position)
        } else {
          false
        }
      }
    };

    if is_drag_start {
      tracing::info!("Window started dragging: {window}");

      window.set_active_drag(Some(ActiveDrag {
        operation: None,
        is_from_floating: matches!(
          window.state(),
          WindowState::Floating(_)
        ),
        #[cfg(target_os = "windows")]
        initial_position: old_frame_position.clone(),
        // The updated frame position is used here instead of the initial
        // frame position due to a quirk on macOS. When we resize an
        // AXUIElement to a value outside the allowed min/max width &
        // height, macOS doesn't actually apply that size. However, it
        // still reports the value we attempted to set until a subsequent
        // `WindowEvent::MovedOrResized` event.
        #[cfg(target_os = "macos")]
        initial_position: frame_position.clone(),
      }));

      #[cfg(target_os = "windows")]
      update_drag_state(&window, &frame_position, state, config)?;

      return Ok(());
    }

    let nearest_monitor = state
      .nearest_monitor(&window.native())
      .context("No nearest monitor.")?;

    // For `HideMethod::PlaceInCorner`, hiding/showing is implemented by
    // repositioning the window. Since the OS won't emit real
    // shown/hidden events in this mode, update `DisplayState` based on
    // whether the window has been moved to the monitor's bottom corner.
    if config.value.general.hide_method == HideMethod::PlaceInCorner {
      let is_in_corner = is_in_corner(
        &frame_position,
        &nearest_monitor.native_properties().working_area,
      );

      // TODO: Consider redrawing if hidden and should be shown, or if
      // shown and should be hidden.
      // TODO: It can be valid for a floating window to be in the corner,
      // in which case, it currently doesn't get updated to
      // `DisplayState::Shown`.
      let display_state = match (window.display_state(), is_in_corner) {
        (DisplayState::Hiding, true) => DisplayState::Hidden,
        (DisplayState::Showing, false) => DisplayState::Shown,
        _ => window.display_state(),
      };

      if display_state != window.display_state() {
        window.set_display_state(display_state);
        return Ok(());
      }
    }

    let should_fullscreen = window.should_fullscreen(
      &nearest_monitor
        .displayed_workspace()
        .context("No workspace.")?,
    )?;

    // Handle a window being maximized or entering fullscreen.
    if is_maximized || should_fullscreen {
      let fullscreen_state = if let WindowState::Fullscreen(
        fullscreen_state,
      ) = window.state()
      {
        fullscreen_state
      } else {
        config
          .value
          .window_behavior
          .state_defaults
          .fullscreen
          .clone()
      };

      update_window_state(
        window.clone(),
        WindowState::Fullscreen(FullscreenStateConfig {
          maximized: is_maximized,
          ..fullscreen_state
        }),
        state,
        config,
      )?;

      // TODO: Consider dequeuing the window from redraw, since the window
      // is already in the correct state. Games are especially sensitive to
      // redraws and are often fullscreen.

      // TODO: Handle a fullscreen window being moved from one monitor to
      // another.

      return Ok(());
    }

    match window.state() {
      WindowState::Fullscreen(_) => {
        // Window is no longer maximized/fullscreen and should be restored.
        tracing::info!("Restoring window from fullscreen: {window}");

        // TODO: Only restore to prev state if it's a floating/tiling
        // state.
        let target_state = window
          .prev_state()
          .unwrap_or(WindowState::default_from_config(&config.value));

        update_window_state(window.clone(), target_state, state, config)?;
      }
      WindowState::Floating(_) => {
        if let WindowContainer::NonTilingWindow(window) = window {
          update_floating_window_position(
            &window,
            frame_position,
            &nearest_monitor,
            state,
          )?;
        }
      }
      _ => {}
    }
  }

  Ok(())
}

// TODO: Move to shared location. `handle_window_moved_or_resized_end.rs`
// also uses this.
pub fn update_floating_window_position(
  window: &NonTilingWindow,
  frame_position: Rect,
  nearest_monitor: &Monitor,
  state: &mut WmState,
) -> anyhow::Result<()> {
  tracing::info!(
    "Updating floating window position: {}",
    window.as_window_container()?
  );

  // Update state with the new location of the floating window.
  window.set_floating_placement(frame_position);
  window.set_has_custom_floating_placement(true);

  let monitor = window.monitor().context("No monitor.")?;

  // Update the window's workspace if it goes out of bounds of its
  // current workspace.
  if monitor.id() != nearest_monitor.id() {
    let updated_workspace = nearest_monitor
      .displayed_workspace()
      .context("Failed to get workspace of nearest monitor.")?;

    tracing::info!(
      "Floating window moved to new workspace: {updated_workspace}",
    );

    window.set_insertion_target(None);

    move_container_within_tree(
      &window.clone().into(),
      &updated_workspace.clone().into(),
      updated_workspace.child_count(),
      state,
    )?;
  }

  Ok(())
}

/// Updates the window operation based on changes in frame position.
///
/// This function determines whether a window is being moved or resized and
/// updates its operation state accordingly. If the window is being moved,
/// it's set to floating mode.
fn update_drag_state(
  window: &WindowContainer,
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
  if is_move && !matches!(window.state(), WindowState::Floating(_)) {
    let move_distance = frame_position
      .center_point()
      .distance_between(&active_drag.initial_position.center_point());

    // Dragging operations on a maximized window can only occur on Windows.
    // The OS immediately restores it while it's being dragged, so we need
    // to update state accordingly without a redraw.
    let is_maximized = matches!(
      window.state(),
      WindowState::Fullscreen(FullscreenStateConfig {
        maximized: true,
        ..
      })
    );

    if move_distance >= 10.0 || is_maximized {
      let parent = window.parent().context("No parent")?;

      let is_fullscreen =
        matches!(window.state(), WindowState::Fullscreen(_))
          && !is_maximized;

      let window = update_window_state(
        window.clone(),
        WindowState::Floating(FloatingStateConfig {
          centered: false,
          ..config.value.window_behavior.state_defaults.floating
        }),
        state,
        config,
      )?;

      // `update_window_state` automatically adds the window for redraw,
      // which we don't want in this case. However, for fullscreen windows,
      // we do actually want it to be resized initially so that it's
      // easier to move around while dragging.
      if !is_fullscreen {
        state
          .pending_sync
          .dequeue_container_from_redraw(window.clone());
      }

      // Flatten the parent split container if it only contains the window.
      // TODO: Consider doing this to `move_container_within_tree`, so that
      // the behavior is consistent.
      if let Some(split_parent) = parent.as_split() {
        if split_parent.child_count() == 1 {
          flatten_split_container(split_parent.clone())?;

          // Hacky fix to redraw siblings after flattening. The parent is
          // queued for redraw from the state change, which gets detached
          // on flatten.
          // TODO: Change `queue_containers_to_redraw` to iterate over its
          // descendant windows and store those instead.
          state
            .pending_sync
            .queue_containers_to_redraw(window.tiling_siblings());
        }
      }
    }
  }

  Ok(())
}

/// Gets whether the window is in the corner of the monitor.
fn is_in_corner(window_frame: &Rect, monitor_rect: &Rect) -> bool {
  // Visible portion of the window used when positioning windows in the
  // monitor's corner. See `platform_sync` for how hidden windows are
  // positioned.
  const VISIBLE_SLIVER_PX: i32 = 1;

  // Allow 1px of leeway.
  let is_left_corner =
    (window_frame.right - VISIBLE_SLIVER_PX - monitor_rect.left).abs()
      <= 1;

  // Allow 1px of leeway.
  let is_right_corner =
    (window_frame.x() + VISIBLE_SLIVER_PX - monitor_rect.right).abs() <= 1;

  // On macOS, the window's title bar is prevented from being positioned
  // outside of monitor's working area, so we need to allow ~50px of
  // vertical leeway.
  let is_bottom_of_monitor =
    (window_frame.y() - monitor_rect.bottom).abs() <= 50;

  (is_left_corner || is_right_corner) && is_bottom_of_monitor
}

#[cfg(test)]
mod tests {
  use wm_platform::Rect;

  use super::is_in_corner;

  #[test]
  fn matches_corner_positions() {
    let monitor = Rect::from_xy(0, 0, 1920, 1080);

    let frame_in_right_corner = Rect::from_xy(1919, 1050, 600, 600);
    assert!(is_in_corner(&frame_in_right_corner, &monitor));

    let frame_in_left_corner = Rect::from_xy(1, 1050, 600, 600);
    assert!(is_in_corner(&frame_in_left_corner, &monitor));
  }

  #[test]
  fn does_not_match_non_corner_positions() {
    let monitor = Rect::from_xy(0, 0, 1920, 1080);
    let frame = Rect::from_xy(100, 100, 800, 600);

    assert!(!is_in_corner(&frame, &monitor));
  }
}
