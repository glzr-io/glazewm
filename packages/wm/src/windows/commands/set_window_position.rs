use anyhow::Context;

use crate::{
  common::{LengthValue, Rect},
  containers::{
    traits::{CommonGetters, PositionGetters},
    WindowContainer,
  },
  windows::{
    traits::WindowGetters, NonTilingWindow, WindowState,
  },
  wm_state::WmState,
};

pub fn set_window_position(
  window: WindowContainer,
  centered: bool,
  target_x_pos: Option<LengthValue>,
  target_y_pos: Option<LengthValue>,
  state: &mut WmState,
) -> anyhow::Result<()> {
  match window {
    WindowContainer::TilingWindow(_) => {
      return Ok(());
    }
    WindowContainer::NonTilingWindow(window) => {
      if matches!(window.state(), WindowState::Floating(_)) {
        set_floating_window_position(
          window,
          centered,
          target_x_pos,
          target_y_pos,
          state,
        )?;
      }
    }
  }

  Ok(())
}

fn set_floating_window_position(
  window: NonTilingWindow,
  centered: bool,
  target_x_pos: Option<LengthValue>,
  target_y_pos: Option<LengthValue>,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let window_rect = window.to_rect()?;

  match centered {
    true => {
      let workspace =
      window.workspace().context("no workspace find.")?;
      window.set_floating_placement(
        window_rect
        .translate_to_center(&workspace.to_rect()?),
      )         
    },
    false => {
      let monitor = window.monitor().context("No monitor")?;
      let monitor_rect = monitor.to_rect()?;
    
      let target_x_pos_px = target_x_pos
      .map(|target_x_pos| target_x_pos.to_px(monitor_rect.x(), None));
      let target_y_pos_px = target_y_pos
      .map(|target_y_pos| target_y_pos.to_px(monitor_rect.y(), None));
    
      let new_x_pos = target_x_pos_px.unwrap_or(window_rect.x());
      let new_y_pos = target_y_pos_px.unwrap_or(window_rect.y());

      window.set_floating_placement(Rect::from_xy(
        new_x_pos,
        new_y_pos,
        window.floating_placement().width(),
        window.floating_placement().height(),
      ));
    }
  }

  state
    .pending_sync
    .containers_to_redraw
    .push(window.clone().into());

  Ok(())
}
