use anyhow::Context;

use crate::{
  common::{platform::Platform, DisplayState},
  containers::{
    traits::{CommonGetters, PositionGetters},
    Container, WindowContainer,
  },
  user_config::UserConfig,
  windows::{traits::WindowGetters, WindowState},
  wm_event::WmEvent,
  wm_state::WmState,
};

pub fn platform_sync(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if state.pending_sync.containers_to_redraw.len() > 0 {
    redraw_containers(state)?;
    state.pending_sync.containers_to_redraw.clear();
  }

  let recent_focused_container = state.recent_focused_container.clone();
  let focused_container =
    state.focused_container().context("No focused container.")?;

  if state.pending_sync.focus_change {
    sync_focus(focused_container.clone(), state)?;
    state.pending_sync.focus_change = false;
  }

  if let Ok(window) = focused_container.as_window_container() {
    apply_window_effects(window, true, config);
  }

  // Get windows that should have the unfocused border applied to them.
  // For the sake of performance, we only update the border of the
  // previously focused window. If the `reset_window_effects` flag is
  // passed, the unfocused border is applied to all unfocused windows.
  let unfocused_windows = match state.pending_sync.reset_window_effects {
    true => state.windows(),
    false => recent_focused_container
      .and_then(|container| container.as_window_container().ok())
      .into_iter()
      .collect(),
  }
  .into_iter()
  .filter(|window| window.id() != focused_container.id());

  for window in unfocused_windows {
    apply_window_effects(window, false, config);
  }

  state.pending_sync.reset_window_effects = false;

  Ok(())
}

pub fn sync_focus(
  focused_container: Container,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let native_window = match focused_container.as_window_container() {
    Ok(window) => window.native(),
    _ => Platform::desktop_window(),
  };

  // Set focus to the given window handle. If the container is a normal
  // window, then this will trigger a `PlatformEvent::WindowFocused` event.
  if Platform::foreground_window() != native_window {
    let _ = native_window.set_foreground();
  }

  // TODO: Change z-index of workspace windows that match the focused
  // container's state. Make sure not to decrease z-index for floating
  // windows that are always on top.

  state.emit_event(WmEvent::FocusChanged {
    focused_container: focused_container.to_dto()?,
  });

  state.recent_focused_container = Some(focused_container);

  Ok(())
}

fn redraw_containers(state: &mut WmState) -> anyhow::Result<()> {
  for window in &state.windows_to_redraw() {
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

    // Restore window if it's minimized/maximized and shouldn't be. This is
    // needed to be able to move and resize it.
    match window.state() {
      // Need to restore window if transitioning from maximized fullscreen
      // to non-maximized fullscreen.
      WindowState::Fullscreen(s) => {
        if !s.maximized && window.native().is_maximized() {
          let _ = window.native().restore();
        }
      }
      // No need to restore window if it'll be minimized. Transitioning
      // from maximized to minimized works without restoring.
      WindowState::Minimized => {}
      _ => {
        if window.native().is_minimized() || window.native().is_maximized()
        {
          let _ = window.native().restore();
        }
      }
    }

    let rect = window.to_rect()?.apply_delta(&window.border_delta());

    let is_visible = match window.display_state() {
      DisplayState::Showing | DisplayState::Shown => true,
      _ => false,
    };

    let _ =
      window
        .native()
        .set_position(&window.state(), is_visible, &rect);

    // When there's a mismatch between the DPI of the monitor and the
    // window, the window might be sized incorrectly after the first move.
    // If we set the position twice, inconsistencies after the first move
    // are resolved.
    if window.has_pending_dpi_adjustment() {
      let _ =
        window
          .native()
          .set_position(&window.state(), is_visible, &rect);

      window.set_has_pending_dpi_adjustment(false);
    }
  }

  Ok(())
}

fn apply_window_effects(
  window: WindowContainer,
  is_focused: bool,
  config: &UserConfig,
) {
  // TODO: Be able to add transparency to windows.

  let enable_borders = match is_focused {
    true => config.value.window_effects.focused_window.border.enabled,
    false => config.value.window_effects.other_windows.border.enabled,
  };

  if enable_borders {
    let border_config = match is_focused {
      true => &config.value.window_effects.focused_window.border,
      false => &config.value.window_effects.other_windows.border,
    }
    .clone();

    _ = window.native().set_border_color(Some(&border_config.color));
  }
}
