use anyhow::Context;

use crate::{
  common::{platform::SetPositionArgs, DisplayState},
  containers::{
    traits::{CommonBehavior, PositionBehavior},
    WindowContainer,
  },
  user_config::UserConfig,
  windows::{traits::WindowBehavior, WindowState},
  wm_state::WmState,
};

pub fn redraw_command(
  state: &mut WmState,
  user_config: UserConfig,
) -> anyhow::Result<&mut WmState> {
  let windows_to_redraw = state.windows_to_redraw();

  // Get windows that are minimized/maximized and shouldn't be.
  let windows_to_restore = windows_to_redraw
    .iter()
    .filter(|window| match window.state() {
      WindowState::Minimized | WindowState::Maximized => false,
      _ => {
        window.native().is_maximized() || window.native().is_minimized()
      }
    })
    .collect::<Vec<_>>();

  // Restore minimized and maximized windows. Needed to be able to move and
  // resize them.
  for window in &windows_to_restore {
    let _ = window.native().restore();
  }

  for window in &windows_to_redraw {
    let workspace = window
      .parent_workspace()
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

    let position_args = get_position_args(
      &window,
      user_config.value.general.show_floating_on_top,
    );

    let _ = window.native().set_position(&position_args);

    // When there's a mismatch between the DPI of the monitor and the
    // window, the window might be sized incorrectly after the first move.
    // If we set the position twice, inconsistencies after the first move
    // are resolved.
    if window.has_pending_dpi_adjustment() {
      let _ = window.native().set_position(&position_args);
      window.set_has_pending_dpi_adjustment(false);
    }
  }

  state.clear_containers_to_redraw();
  Ok(state)
}

fn get_position_args(
  window: &WindowContainer,
  show_floating_on_top: bool,
) -> SetPositionArgs {
  // Avoid adjusting the borders of non-tiling windows. Otherwise the
  // window will increase in size from its original placement.
  let rect = match window.state() {
    WindowState::Tiling => {
      window.to_rect().apply_delta(window.border_delta())
    }
    _ => window.to_rect(),
  };

  SetPositionArgs {
    window_handle: window.native().handle,
    visible: match window.display_state() {
      DisplayState::Showing | DisplayState::Shown => true,
      _ => false,
    },
    show_on_top: match window.state() {
      WindowState::Floating => show_floating_on_top,
      _ => false,
    },
    move_and_resize: match window.state() {
      WindowState::Floating | WindowState::Tiling => true,
      _ => false,
    },
    rect,
  }
}
