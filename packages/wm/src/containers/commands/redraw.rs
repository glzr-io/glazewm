use anyhow::Context;

use crate::{
  common::DisplayState,
  containers::traits::{CommonGetters, PositionGetters},
  windows::{traits::WindowGetters, WindowState},
  wm_state::WmState,
};

pub fn redraw(state: &mut WmState) -> anyhow::Result<()> {
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

  state.clear_containers_to_redraw();

  Ok(())
}
