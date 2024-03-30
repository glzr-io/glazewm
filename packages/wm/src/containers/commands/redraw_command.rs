use std::ptr::null_mut;

use crate::{
  user_config::UserConfig, windows::WindowState, wm_state::WmState,
};

pub struct RedrawCommand;

pub fn redraw_handler(
  command: RedrawCommand,
  state: &mut WmState,
  user_config: UserConfig,
) -> anyhow::Result<&mut WmState> {
  let windows_to_redraw = state.windows_to_redraw();

  // Get windows that are minimized/maximized and shouldn't be.
  let windows_to_restore = windows_to_redraw
    .iter()
    .filter(|window| {
      window.state == WindowState::Minimized
        && !window.native().is_minimized()
        || window.state == WindowState::Maximized
          && window.native().is_maximized()
    })
    .collect::<Vec<_>>();

  // Restore minimized and maximized windows. Needed to be able to move and
  // resize them.
  for window in &windows_to_restore {
    let _ = window.native().restore();
  }

  // Get z-order to set for floating windows.
  let should_show_on_top = self
    .user_config_service
    .general_config()
    .show_floating_on_top;

  for window in &windows_to_redraw {
    let workspace = window
      .ancestors()
      .find_map(|ancestor| ancestor.as_workspace())
      .context("Window has no workspace.")?;

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

    window.native().set_position(position);

    // When there's a mismatch between the DPI of the monitor and the window,
    // `SetWindowPos` might size the window incorrectly. By calling `SetWindowPos`
    // twice, inconsistencies after the first move are resolved.
    if window.has_pending_dpi_adjustment() {
      window.native().set_position(position);
      window.set_has_pending_dpi_adjustment(false);
    }
  }

  state.clear_containers_to_redraw();
  Ok(state)
}

fn set_window_position(&self, window: &Window) {
  let default_flags = SetWindowPosFlags::FRAME_CHANGED
    | SetWindowPosFlags::NO_ACTIVATE
    | SetWindowPosFlags::NO_COPY_BITS
    | SetWindowPosFlags::NO_SEND_CHANGING
    | SetWindowPosFlags::ASYNC_WINDOW_POS;

  let workspace = window
    .ancestors()
    .find_map(|ancestor| ancestor.as_workspace())
    .context("Window has no workspace.")?;

  let is_workspace_displayed = workspace.is_displayed();

  // Show or hide the window depending on whether the workspace is displayed.
  let default_flags = if is_workspace_displayed {
    default_flags | SetWindowPosFlags::SHOW_WINDOW
  } else {
    default_flags | SetWindowPosFlags::HIDE_WINDOW
  };

  let default_flags = if window.is::<MaximizedWindow>() {
    default_flags | SetWindowPosFlags::NO_SIZE | SetWindowPosFlags::NO_MOVE
  } else {
    default_flags
  };

  // Transition display state depending on whether window will be shown/hidden.
  window.set_display_state(
    match (window.display_state(), is_workspace_displayed) {
      (DisplayState::Hidden | DisplayState::Hiding, true) => {
        DisplayState::Showing
      }
      (DisplayState::Shown | DisplayState::Showing, false) => {
        DisplayState::Hiding
      }
      _ => window.display_state(),
    },
  );

  if window.is::<TilingWindow>() {
    set_window_pos(
      window.handle(),
      null_mut(),
      window.x() - window.border_delta().left,
      window.y() - window.border_delta().top,
      window.width()
        + window.border_delta().left
        + window.border_delta().right,
      window.height()
        + window.border_delta().top
        + window.border_delta().bottom,
      default_flags,
    );
    return;
  }

  // Get z-order to set for floating windows.
  let should_show_on_top = self
    .user_config_service
    .general_config()
    .show_floating_on_top;

  let floating_z_order = if should_show_on_top {
    ZOrderFlags::TopMost
  } else {
    ZOrderFlags::NoTopMost
  };

  // Avoid adjusting the borders of floating windows. Otherwise the window will
  // increase in size from its original placement.
  set_window_pos(
    window.handle(),
    floating_z_order as _,
    window.x(),
    window.y(),
    window.width(),
    window.height(),
    default_flags,
  );
}
