use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use tokio::task;
use tracing::warn;
use uuid::Uuid;
use wm_common::{
  CornerStyle, CursorJumpTrigger, DisplayState, HideMethod, OpacityValue,
  UniqueExt, WindowEffectConfig, WindowState, WmEvent,
};
use wm_platform::{Platform, ZOrder};

use crate::{
  models::{Container, WindowContainer},
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn platform_sync(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // Skip platform sync when the WM is paused.
  if state.is_paused {
    // Clear containers to redraw to avoid leaking memory.
    state.pending_sync.containers_to_redraw.clear();
    return Ok(());
  }

  let focused_container =
    state.focused_container().context("No focused container.")?;

  // Keep reference to the original focused container.
  let recent_focused_container = state.recent_focused_container.clone();

  if state.pending_sync.focus_change {
    sync_focus(&focused_container, state)?;
  }

  if !state.pending_sync.containers_to_redraw.is_empty()
    || state.pending_sync.focus_change
  {
    redraw_containers(
      &focused_container,
      recent_focused_container.as_ref(),
      state,
      config,
    )?;
  }

  if state.pending_sync.cursor_jump
    && config.value.general.cursor_jump.enabled
  {
    jump_cursor(focused_container.clone(), state, config)?;
  }

  if state.pending_sync.focus_change
    || state.pending_sync.reset_window_effects
  {
    if let Ok(window) = focused_container.as_window_container() {
      apply_window_effects(&window, true, config);
    }
  }

  if state.pending_sync.reset_window_effects {
    // Get windows that should have the unfocused border applied to them.
    // For the sake of performance, we only update the border of the
    // previously focused window. If the `reset_window_effects` flag is
    // passed, the unfocused border is applied to all unfocused windows.
    let unfocused_windows = if state.pending_sync.reset_window_effects {
      state.windows()
    } else {
      let prev_focused_window = recent_focused_container
        .as_ref()
        .and_then(|container| container.as_window_container().ok());

      prev_focused_window.into_iter().collect()
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
    if let Err(err) = native_window.set_foreground() {
      warn!("Failed to set foreground window: {}", err);
    }
  }

  // TODO: Change z-index of workspace windows that match the focused
  // container's state. Make sure not to decrease z-index for floating
  // windows that are always on top.

  state.emit_event(WmEvent::FocusChanged {
    focused_container: focused_container.to_dto()?,
  });

  state.recent_focused_container = Some(focused_container.clone());

  Ok(())
}

fn window_z_orders(
  windows_to_bring_to_front: &[WindowContainer],
  windows_to_update: &[&WindowContainer],
) -> HashMap<Uuid, ZOrder> {
  let mut z_orders = HashMap::new();
  let mut next_front_window = None;

  for window in windows_to_update {
    let should_bring_to_front = windows_to_bring_to_front.contains(window);

    // Whether the window should be shown above all other windows.
    let z_order = match window.state() {
      WindowState::Floating(config) if config.shown_on_top => {
        ZOrder::TopMost
      }
      WindowState::Fullscreen(config) if config.shown_on_top => {
        ZOrder::TopMost
      }
      _ if should_bring_to_front => {
        // Find the next window above this one in the focus order.
        if let Some(prev) = next_front_window {
          next_front_window = Some(window);
          ZOrder::AfterWindow(prev.native().handle)
        } else {
          next_front_window = Some(window);
          ZOrder::Top
        }
      }
      _ => ZOrder::Normal,
    };

    z_orders.insert(window.id(), z_order);
  }

  z_orders
}

fn windows_to_bring_to_front(
  focused_container: &Container,
  recent_focused_container: Option<&Container>,
  state: &WmState,
) -> anyhow::Result<Vec<WindowContainer>> {
  if !state.pending_sync.focus_change {
    return Ok(vec![]);
  }

  let Some(focused_window) = focused_container.as_window_container().ok()
  else {
    return Ok(vec![]);
  };

  let focused_workspace =
    focused_container.workspace().context("No workspace.")?;

  let prev_focused_window = recent_focused_container
    .and_then(|container| container.as_window_container().ok());

  // Check if we need to reorder based on previous focus.
  // let should_bring_front = prev_focused.is_none_or(|prev_window| {
  //   // Bring to front if either:
  //   // 1. Window state has changed
  //   // 2. Window moved to different workspace
  //   focused_state
  //     .as_ref()
  //     .map(|current_state| {
  //       prev_window.state() != *current_state
  //         || prev_window.workspace().map(|w| w.id())
  //           != Some(focused_workspace.id())
  //     })
  //     .unwrap_or(false);

  //   Some(prev_window.state()) == focused_state
  // });

  // Check if we need to reorder based on previous focus.
  let should_bring_to_front = match prev_focused_window {
    Some(prev_window) => {
      // Bring to front if either:
      // 1. Window state has changed.
      // 2. Window moved to different workspace.
      let prev_workspace_id =
        prev_window.workspace().map(|workspace| workspace.id());

      focused_window.state() != prev_window.state()
        || prev_workspace_id != Some(focused_workspace.id())
    }
    None => true,
  };

  // Get windows that match the focused container's state.
  let windows_to_bring_to_front = if should_bring_to_front {
    focused_workspace
      .descendants()
      .filter_map(|descendant| descendant.as_window_container().ok())
      .filter(|window| {
        let is_floating_or_tiling = matches!(
          window.state(),
          WindowState::Floating(_) | WindowState::Tiling
        );

        // Only if floating or tiling and the window state matches the
        // focused state.
        is_floating_or_tiling
          && window.state().is_same_state(&focused_window.state())
      })
      .collect::<Vec<_>>()
  } else {
    vec![focused_window]
  };

  Ok(windows_to_bring_to_front)
}

fn redraw_containers(
  focused_container: &Container,
  recent_focused_container: Option<&Container>,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let windows_to_redraw = state.windows_to_redraw();
  let windows_to_bring_to_front = windows_to_bring_to_front(
    focused_container,
    recent_focused_container,
    state,
  )?;

  let mut windows_to_update = windows_to_redraw
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
  windows_to_update.sort_by_key(|window| {
    descendant_focus_order
      .iter()
      .position(|order| order.id() == window.id())
  });

  let z_orders =
    window_z_orders(&windows_to_bring_to_front, &windows_to_update);

  for window in windows_to_update {
    // Whether the window should be shown above all other windows.
    let z_order = z_orders.get(&window.id()).unwrap_or(&ZOrder::Normal);

    // Set the z-order of the window and skip updating it's position if the
    // window only requires a z-order change.
    if windows_to_bring_to_front.contains(window)
      && !windows_to_redraw.contains(window)
    {
      tracing::info!("Setting window z-order: {window}");

      if let Err(err) = window.native().set_z_order(z_order) {
        warn!("Failed to set window z-order: {}", err);
      }

      continue;
    }

    tracing::info!("Setting window position: {window}");

    let workspace =
      window.workspace().context("Window has no workspace.")?;

    // Transition display state depending on whether window will be
    // shown or hidden.
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

    if let Err(err) = window.native().set_position(
      &window.state(),
      &rect,
      z_order,
      is_visible,
      &config.value.general.hide_method,
      window.has_pending_dpi_adjustment(),
    ) {
      warn!("Failed to set window position: {}", err);
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
  };

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
  _ = window
    .native()
    .set_opacity(if effect_config.transparency.enabled {
      &effect_config.transparency.opacity
    } else {
      // This code is only reached if the transparency effect is only
      // enabled in one of the two window effect configurations. In
      // this case, reset the opacity to default.
      &OpacityValue {
        amount: 255,
        is_delta: false,
      }
    });
}
