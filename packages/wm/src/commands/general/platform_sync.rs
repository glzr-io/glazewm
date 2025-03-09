use std::time::Duration;

use anyhow::Context;
use tokio::task;
use tracing::{info, warn};
use wm_common::{
  CornerStyle, CursorJumpTrigger, DisplayState, HideMethod, OpacityValue,
  TilingDirection, UniqueExt, WindowEffectConfig, WindowState, WmEvent,
};
use wm_platform::{Platform, ZOrder};

use crate::{
  models::{Container, DirectionContainer, WindowContainer},
  traits::{
    CommonGetters, PositionGetters, TilingDirectionGetters, WindowGetters,
  },
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

  // Process all regular windows first
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

    windows.sort_by_key(|window| {
      descendant_focus_order
        .iter()
        .position(|order| order.id() == window.id())
    });

    windows
  };

  // FIRST PASS: Process all windows normally for positioning
  for window in windows_to_update.iter().rev() {
    let should_bring_to_front = windows_to_bring_to_front.contains(window);

    // Check if this window is in an accordion
    let is_in_accordion = if let Some(parent) = window.parent() {
      if let Ok(direction_parent) = parent.as_direction_container() {
        matches!(
          direction_parent.tiling_direction(),
          TilingDirection::HorizontalAccordion
            | TilingDirection::VerticalAccordion
        )
      } else {
        false
      }
    } else {
      false
    };

    let workspace =
      window.workspace().context("Window has no workspace.")?;

    // Use a temporary normal z-order for all windows
    let z_order = match window.state() {
      WindowState::Floating(config) if config.shown_on_top => {
        ZOrder::TopMost
      }
      WindowState::Fullscreen(config) if config.shown_on_top => {
        ZOrder::TopMost
      }
      _ => ZOrder::Normal,
    };

    // Skip positioning update if only z-order change needed
    if should_bring_to_front
      && !windows_to_redraw.contains(window)
      && !is_in_accordion
    {
      info!("Updating window z-order: {window}");

      if let Err(err) = window.native().set_z_order(&z_order) {
        warn!("Failed to set window z-order: {}", err);
      }

      continue;
    }

    // Normal display state and positioning update
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

    let rect = window
      .to_rect()?
      .apply_delta(&window.total_border_delta()?, None);

    let is_visible = matches!(
      window.display_state(),
      DisplayState::Showing | DisplayState::Shown
    );

    // For position updates, use a temporary Normal z-order
    if let Err(err) = window.native().set_position(
      &window.state(),
      &rect,
      &z_order,
      is_visible,
      &config.value.general.hide_method,
      window.has_pending_dpi_adjustment(),
    ) {
      warn!("Failed to set window position: {}", err);
    }

    // Handle fullscreen transitions
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

    // Handle taskbar visibility
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

  // SECOND PASS: Handle z-order for accordion containers
  // Find all accordion containers
  let accordion_containers: Vec<DirectionContainer> = state
    .root_container
    .descendants()
    .filter_map(|container| container.as_direction_container().ok())
    .filter(|container| {
      matches!(
        container.tiling_direction(),
        TilingDirection::HorizontalAccordion
          | TilingDirection::VerticalAccordion
      )
    })
    .collect();

  // Process z-order for each accordion
  for accordion in accordion_containers {
    // Get all windows from this accordion
    let accordion_windows: Vec<WindowContainer> = accordion
      .children()
      .into_iter()
      .filter_map(|c| c.as_window_container().ok())
      .collect();

    if accordion_windows.is_empty() {
      continue;
    }

    // Find the focused window in this accordion
    let has_focus = accordion_windows
      .iter()
      .any(|window| window.id() == focused_container.id());

    if !has_focus {
      continue;
    }

    // Force focused window to TOP z-order
    for window in accordion_windows.iter() {
      if window.id() == focused_container.id() {
        // Extra call to ensure this window is on top
        if let Err(err) = window.native().set_z_order(&ZOrder::Top) {
          warn!("Failed to set focused accordion window z-order: {}", err);
        }
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

      let cursor_monitor =
        state.monitor_at_point(&Platform::mouse_position()?);

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
