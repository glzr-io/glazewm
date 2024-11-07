use anyhow::Context;

use crate::{
  common::Rect,
  containers::{
    traits::{CommonGetters, PositionGetters},
    WindowContainer,
  },
  windows::{
    traits::WindowGetters, WindowState,
  },
  wm_state::WmState,
};

pub fn set_window_position(
  window: WindowContainer,
  target_x_pos: Option<i32>,
  target_y_pos: Option<i32>,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if matches!(window.state(), WindowState::Floating(_)) {
    let window_rect = window.floating_placement();
    let new_x_pos = target_x_pos.unwrap_or(window_rect.x());
    let new_y_pos = target_y_pos.unwrap_or(window_rect.y());
    window.set_floating_placement(Rect::from_xy(
      new_x_pos,
      new_y_pos,
      window.floating_placement().width(),
      window.floating_placement().height(),
    ));
  
    state
      .pending_sync
      .containers_to_redraw
      .push(window.clone().into());
  }

  Ok(())
}

pub fn set_window_position_to_center(
  window: WindowContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if matches!(window.state(), WindowState::Floating(_)) {
    let window_rect = window.floating_placement();
      let workspace =
      window.workspace().context("no workspace find.")?;
      window.set_floating_placement(
        window_rect
        .translate_to_center(&workspace.to_rect()?),
      );

    state
    .pending_sync
    .containers_to_redraw
    .push(window.clone().into());    
  }

  Ok(())
}