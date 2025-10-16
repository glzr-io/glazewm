use std::time::Duration;

use anyhow::Context;
use tokio::task;
use tracing::{info, warn};
use wm_common::{
  CornerStyle, CursorJumpTrigger, DisplayState, EasingFunction, HideMethod,
  OpacityValue, UniqueExt, WindowEffectConfig, WindowState, WmEvent,
};
use wm_platform::{Platform, ZOrder};

use crate::{
  animation_state::WindowAnimationState,
  models::{Container, WindowContainer},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn platform_sync(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_container =
    state.focused_container().context("No focused container.")?;

  if state.pending_sync.needs_focus_update() {
    sync_focus(&focused_container, state)?;
  }

  if !state.pending_sync.containers_to_redraw().is_empty()
    || !state.pending_sync.workspaces_to_reorder().is_empty()
  {
    redraw_containers(&focused_container, state, config)?;
  }

  if state.pending_sync.needs_cursor_jump()
    && config.value.general.cursor_jump.enabled
  {
    jump_cursor(focused_container.clone(), state, config)?;
  }

  if state.pending_sync.needs_focused_effect_update()
    || state.pending_sync.needs_all_effects_update()
  {
    // Keep reference to the previous window that had focus effects
    // applied.
    let prev_effects_window = state.prev_effects_window.clone();

    if let Ok(window) = focused_container.as_window_container() {
      apply_window_effects(&window, true, config);
      state.prev_effects_window = Some(window.clone());
    } else {
      state.prev_effects_window = None;
    }

    // Get windows that should have the unfocused border applied to them.
    // For the sake of performance, we only update the border of the
    // previously focused window. If the `reset_window_effects` flag is
    // passed, the unfocused border is applied to all unfocused windows.
    let unfocused_windows =
      if state.pending_sync.needs_all_effects_update() {
        state.windows()
      } else {
        prev_effects_window.into_iter().collect()
      }
      .into_iter()
      .filter(|window| window.id() != focused_container.id());

    for window in unfocused_windows {
      apply_window_effects(&window, false, config);
    }
  }

  state.pending_sync.clear();

  Ok(())
}

fn sync_focus(
  focused_container: &Container,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let native_window = match focused_container.as_window_container() {
    Ok(window) => window.native().clone(),
    _ => Platform::desktop_window(),
  };

  // Set focus to the given window handle. If the container is a normal
  // window, then this will trigger a `PlatformEvent::WindowFocused` event.
  if Platform::foreground_window() != native_window {
    if let Ok(window) = focused_container.as_window_container() {
      info!("Setting focus to window: {window}");
    } else {
      info!("Setting focus to the desktop window.");
    }

    if let Err(err) = native_window.set_foreground() {
      warn!("Failed to set foreground window: {}", err);
    }
  }

  state.emit_event(WmEvent::FocusChanged {
    focused_container: focused_container.to_dto()?,
  });

  Ok(())
}

/// Finds windows that should be brought to the top of their workspace's
/// z-order.
///
/// Windows are brought to front if they match the focused window's state
/// (floating/tiling) and any of these conditions are met:
///  * Focus has changed to a different window.
///  * Focused window's state has changed (e.g. tiling -> floating).
///  * Focused window has moved to a different workspace.
fn windows_to_bring_to_front(
  focused_container: &Container,
  state: &WmState,
) -> anyhow::Result<Vec<WindowContainer>> {
  let focused_workspace =
    focused_container.workspace().context("No workspace.")?;

  // Add focused workspace if there's been a focus change.
  let workspaces_to_reorder = state
    .pending_sync
    .workspaces_to_reorder()
    .iter()
    .chain(
      state
        .pending_sync
        .needs_focus_update()
        .then_some(&focused_workspace),
    )
    .unique_by(|workspace| workspace.id());

  // Bring forward windows that match the focused state. Only do this for
  // tiling/floating windows.
  let windows_to_bring_to_front = workspaces_to_reorder
    .flat_map(|workspace| {
      let focused_descendant = workspace
        .descendant_focus_order()
        .next()
        .and_then(|container| container.as_window_container().ok());

      match focused_descendant {
        Some(focused_descendant) => workspace
          .descendants()
          .filter_map(|descendant| descendant.as_window_container().ok())
          .filter(|window| {
            let is_floating_or_tiling = matches!(
              window.state(),
              WindowState::Floating(_) | WindowState::Tiling
            );

            is_floating_or_tiling
              && window.state().is_same_state(&focused_descendant.state())
          })
          .collect(),
        None => vec![],
      }
    })
    .collect::<Vec<_>>();

  Ok(windows_to_bring_to_front)
}

#[allow(clippy::too_many_lines)]
fn redraw_containers(
  focused_container: &Container,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let windows_to_redraw = state.windows_to_redraw();
  let windows_to_bring_to_front =
    windows_to_bring_to_front(focused_container, state)?;

  let windows_to_update = {
    let mut windows = windows_to_redraw
      .iter()
      .chain(&windows_to_bring_to_front)
      .unique_by(|window| window.id())
      .collect::<Vec<_>>();

    let descendant_focus_order = state
      .root_container
      .descendant_focus_order()
      .collect::<Vec<_>>();

    // Sort the windows to update by their focus order. The most recently
    // focused window will be updated first.
    windows.sort_by_key(|window| {
      descendant_focus_order
        .iter()
        .position(|order| order.id() == window.id())
    });

    windows
  };

  for window in windows_to_update.iter().rev() {
    let should_bring_to_front = windows_to_bring_to_front.contains(window);

    let workspace =
      window.workspace().context("Window has no workspace.")?;

    // Whether the window should be shown above all other windows.
    let z_order = match window.state() {
      WindowState::Floating(config) if config.shown_on_top => {
        ZOrder::TopMost
      }
      WindowState::Fullscreen(config) if config.shown_on_top => {
        ZOrder::TopMost
      }
      _ if should_bring_to_front => {
        let focused_descendant = workspace
          .descendant_focus_order()
          .next()
          .and_then(|container| container.as_window_container().ok());

        if let Some(focused_descendant) = focused_descendant {
          if window.id() == focused_descendant.id() {
            ZOrder::Normal
          } else {
            ZOrder::AfterWindow(focused_descendant.native().handle)
          }
        } else {
          ZOrder::Normal
        }
      }
      _ => ZOrder::Normal,
    };

    // Set the z-order of the window and skip updating it's position if the
    // window only requires a z-order change.
    if should_bring_to_front && !windows_to_redraw.contains(window) {
      info!("Updating window z-order: {window}");

      if let Err(err) = window.native().set_z_order(&z_order) {
        warn!("Failed to set window z-order: {}", err);
      }

      continue;
    }

    // Transition display state depending on whether window will be
    // shown or hidden.
    let prev_display_state = window.display_state();
    window.set_display_state(
      match (window.display_state(), workspace.is_displayed()) {
        (DisplayState::Hidden | DisplayState::Hiding, true) => {
          DisplayState::Showing
        }
        (DisplayState::Shown | DisplayState::Showing, false) => {
          DisplayState::Hiding
        }
        _ => window.display_state(),
      },
    );

    let target_rect = window
      .to_rect()?
      .apply_delta(&window.total_border_delta()?, None);

    let is_visible = matches!(
      window.display_state(),
      DisplayState::Showing | DisplayState::Shown
    );

    // Determine if we should animate this window
    let animations_enabled = config.value.animations.enabled;
    let is_opening = matches!(
      (prev_display_state, window.display_state()),
      (DisplayState::Hidden, DisplayState::Showing)
    );
    let is_closing = matches!(
      window.display_state(),
      DisplayState::Hiding
    );

    // Determine the rect and opacity to use
    let (rect_to_use, opacity_override) = if animations_enabled {
      // Get the previous target position before updating
      let previous_target = state.window_target_positions.get(&window.id()).cloned();

      // Update the stored target position
      state.window_target_positions.insert(window.id(), target_rect.clone());

      // Check if there's already an animation for this window
      let existing_animation = state.animation_manager.get_animation(&window.id());

      // Decide whether to start a new animation
      let should_start_new_animation = if is_opening && config.value.animations.window_open.enabled {
        existing_animation.is_none()
      } else if is_closing && config.value.animations.window_close.enabled {
        existing_animation.is_none()
      } else if !is_opening && !is_closing && config.value.animations.window_movement.enabled {
        if let Some(anim) = existing_animation {
          // Don't restart animations that are completing or already at target
          if anim.is_complete() {
            false
          } else {
            // Only start new animation if target has changed significantly
            let target_distance = (anim.target_rect.x() - target_rect.x()).abs() +
                                 (anim.target_rect.y() - target_rect.y()).abs() +
                                 (anim.target_rect.width() - target_rect.width()).abs() +
                                 (anim.target_rect.height() - target_rect.height()).abs();
            target_distance > 5
          }
        } else if let Some(ref prev_target) = previous_target {
          // Compare PREVIOUS target to NEW target, not current position to target
          let distance = (prev_target.x() - target_rect.x()).abs() +
                         (prev_target.y() - target_rect.y()).abs() +
                         (prev_target.width() - target_rect.width()).abs() +
                         (prev_target.height() - target_rect.height()).abs();
          distance > 10
        } else {
          // First time seeing this window, no animation needed
          false
        }
      } else {
        false
      };

      // Start new animation if needed
      if should_start_new_animation {
        if is_opening {
          let animation = WindowAnimationState::new_open(
            target_rect.clone(),
            &config.value.animations.window_open,
          );
          state.animation_manager.start_animation(window.id(), animation);
        } else if is_closing {
          let animation = WindowAnimationState::new_close(
            target_rect.clone(),
            &config.value.animations.window_close,
          );
          state.animation_manager.start_animation(window.id(), animation);
        } else if let Some(prev_target) = previous_target.clone() {
          // Detect operation type and calculate adaptive timing
          let prev_area = prev_target.width() * prev_target.height();
          let target_area = target_rect.width() * target_rect.height();
          let area_change_ratio = if prev_area > 0 {
            ((target_area as f32 / prev_area as f32) - 1.0).abs()
          } else {
            1.0
          };

          let is_expansion = target_area > prev_area;
          let is_shrinking = target_area < prev_area;

          // Create custom animation config based on operation type and size change
          let mut animation_config = config.value.animations.window_movement.clone();

          // Determine base duration based on operation type
          let base_duration = if is_expansion {
            80  // Fast for expansions to minimize black areas
          } else if is_shrinking {
            100  // Slightly faster for shrinking
          } else {
            150  // Normal speed for pure movement (< 1% area change)
          };

          // Scale duration based on magnitude of size change
          let adaptive_duration = if area_change_ratio < 0.2 {
            // Small change (< 20% area): Full animation
            base_duration
          } else if area_change_ratio < 0.5 {
            // Medium change (20-50%): 70% of duration
            (base_duration as f32 * 0.7) as u32
          } else {
            // Large change (> 50%): 50% of duration, very fast
            (base_duration as f32 * 0.5) as u32
          };

          animation_config.duration_ms = adaptive_duration.max(40); // Never less than 40ms

          // Use better easing for expansions
          if is_expansion {
            animation_config.easing = EasingFunction::EaseOutCubic; // Fast start, slow end
          } else if is_shrinking {
            animation_config.easing = EasingFunction::EaseInOutCubic; // Smooth both ways
          }

          // Use previous target as start position for movement animation
          let animation = WindowAnimationState::new_movement(
            prev_target,
            target_rect.clone(),
            &animation_config,
          );
          state.animation_manager.start_animation(window.id(), animation);
        }
      }

      // Get the current animation state (re-fetch after potentially starting new animation)
      if let Some(animation) = state.animation_manager.get_animation(&window.id()) {
        (animation.current_rect(), animation.current_opacity())
      } else {
        (target_rect.clone(), None)
      }
    } else {
      (target_rect.clone(), None)
    };

    info!("Updating window position: {window}");

    if let Err(err) = window.native().set_position(
      &window.state(),
      &rect_to_use,
      &z_order,
      is_visible,
      &config.value.general.hide_method,
      window.has_pending_dpi_adjustment(),
    ) {
      warn!("Failed to set window position: {}", err);
    }

    // Apply opacity if there's an animation with fade
    if let Some(opacity) = opacity_override {
      if let Err(err) = window.native().set_transparency(&opacity) {
        warn!("Failed to set window opacity during animation: {}", err);
      }
    }

    // Whether the window is either transitioning to or from fullscreen.
    // TODO: This check can be improved since `prev_state` can be
    // fullscreen without it needing to be marked as not fullscreen.
    let is_transitioning_fullscreen =
      match (window.prev_state(), window.state()) {
        (Some(_), WindowState::Fullscreen(s)) if !s.maximized => true,
        (Some(WindowState::Fullscreen(_)), _) => true,
        _ => false,
      };

    if is_transitioning_fullscreen {
      if let Err(err) = window.native().mark_fullscreen(matches!(
        window.state(),
        WindowState::Fullscreen(_)
      )) {
        warn!("Failed to mark window as fullscreen: {}", err);
      }
    }

    // Skip setting taskbar visibility if the window is hidden (has no
    // effect). Since cloaked windows are normally always visible in the
    // taskbar, we only need to set visibility if `show_all_in_taskbar` is
    // `false`.
    if config.value.general.hide_method == HideMethod::Cloak
      && !config.value.general.show_all_in_taskbar
      && matches!(
        window.display_state(),
        DisplayState::Showing | DisplayState::Hiding
      )
    {
      if let Err(err) = window.native().set_taskbar_visibility(is_visible)
      {
        warn!("Failed to set taskbar visibility: {}", err);
      }
    }
  }

  Ok(())
}

fn jump_cursor(
  focused_container: Container,
  state: &WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let cursor_jump = &config.value.general.cursor_jump;

  let jump_target = match cursor_jump.trigger {
    CursorJumpTrigger::WindowFocus => Some(focused_container),
    CursorJumpTrigger::MonitorFocus => {
      let target_monitor =
        focused_container.monitor().context("No monitor.")?;

      let cursor_monitor = Platform::mouse_position()
        .ok()
        .and_then(|pos| state.monitor_at_point(&pos));

      // Jump to the target monitor if the cursor is not already on it.
      cursor_monitor
        .filter(|monitor| monitor.id() != target_monitor.id())
        .map(|_| target_monitor.into())
    }
  };

  if let Some(jump_target) = jump_target {
    let center = jump_target.to_rect()?.center_point();

    if let Err(err) = Platform::set_cursor_pos(center.x, center.y) {
      warn!("Failed to set cursor position: {}", err);
    }
  }

  Ok(())
}

fn apply_window_effects(
  window: &WindowContainer,
  is_focused: bool,
  config: &UserConfig,
) {
  let window_effects = &config.value.window_effects;

  let effect_config = if is_focused {
    &window_effects.focused_window
  } else {
    &window_effects.other_windows
  };

  // Skip if both focused + non-focused window effects are disabled.
  if window_effects.focused_window.border.enabled
    || window_effects.other_windows.border.enabled
  {
    apply_border_effect(window, effect_config);
  }

  if window_effects.focused_window.hide_title_bar.enabled
    || window_effects.other_windows.hide_title_bar.enabled
  {
    apply_hide_title_bar_effect(window, effect_config);
  }

  if window_effects.focused_window.corner_style.enabled
    || window_effects.other_windows.corner_style.enabled
  {
    apply_corner_effect(window, effect_config);
  }

  if window_effects.focused_window.transparency.enabled
    || window_effects.other_windows.transparency.enabled
  {
    apply_transparency_effect(window, effect_config);
  }
}

fn apply_border_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  let border_color = if effect_config.border.enabled {
    Some(&effect_config.border.color)
  } else {
    None
  };

  _ = window.native().set_border_color(border_color);

  let native = window.native().clone();
  let border_color = border_color.cloned();

  // Re-apply border color after a short delay to better handle
  // windows that change it themselves.
  task::spawn(async move {
    tokio::time::sleep(Duration::from_millis(50)).await;
    _ = native.set_border_color(border_color.as_ref());
  });
}

fn apply_hide_title_bar_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  _ = window
    .native()
    .set_title_bar_visibility(!effect_config.hide_title_bar.enabled);
}

fn apply_corner_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  let corner_style = if effect_config.corner_style.enabled {
    &effect_config.corner_style.style
  } else {
    &CornerStyle::Default
  };

  _ = window.native().set_corner_style(corner_style);
}

fn apply_transparency_effect(
  window: &WindowContainer,
  effect_config: &WindowEffectConfig,
) {
  let transparency = if effect_config.transparency.enabled {
    &effect_config.transparency.opacity
  } else {
    // Reset the transparency to default.
    &OpacityValue::from_alpha(u8::MAX)
  };

  _ = window.native().set_transparency(transparency);
}
