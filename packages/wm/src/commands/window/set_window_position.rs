use anyhow::Context;
use wm_common::{Rect, WindowState};

use crate::{
  models::WindowContainer,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  wm_state::WmState,
};

pub enum WindowPositionTarget {
  Centered,
  Coordinates(Option<i32>, Option<i32>),
}

pub fn set_window_position(
  window: WindowContainer,
  target: WindowPositionTarget,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if matches!(window.state(), WindowState::Floating(_)) {
    let placement = window.floating_placement();

    let new_placement = match target {
      WindowPositionTarget::Centered => placement.translate_to_center(
        &window.workspace().context("No workspace.")?.to_rect()?,
      ),
      WindowPositionTarget::Coordinates(target_x, target_y) => {
        Rect::from_xy(
          target_x.unwrap_or(placement.x()),
          target_y.unwrap_or(placement.y()),
          placement.width(),
          placement.height(),
        )
      }
    };

    window.set_floating_placement(new_placement);
    state.pending_sync.containers_to_redraw.push(window.into());
  }

  Ok(())
}
