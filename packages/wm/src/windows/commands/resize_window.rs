use anyhow::Context;

use crate::{
  common::LengthValue,
  containers::{
    traits::{CommonGetters, PositionGetters, TilingSizeGetters},
    WindowContainer,
  },
  wm_state::WmState,
};

use super::set_window_size;

pub fn resize_window(
  window: WindowContainer,
  width_delta: Option<LengthValue>,
  height_delta: Option<LengthValue>,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let monitor = window.monitor().context("No monitor")?;
  let monitor_rect = monitor.to_rect()?;
  let window_rect = window.to_rect()?;

  let target_width = match width_delta {
    Some(delta) => {
      let parent_width = match window.as_tiling_container() {
        Ok(tiling_window) => tiling_window
          .container_to_resize(true)?
          .and_then(|container| container.parent())
          .and_then(|parent| parent.width().ok())
          .map(|parent_width| {
            parent_width
              - tiling_window.inner_gap().to_pixels(monitor_rect.width())
                * tiling_window.tiling_siblings().count() as i32
          }),
        _ => window.parent().and_then(|parent| parent.width().ok()),
      };

      parent_width.map(|parent_width| {
        let delta_px = delta.to_pixels(parent_width);
        window_rect.width() + delta_px
      })
    }
    _ => None,
  };

  let target_height = match height_delta {
    Some(delta) => {
      let parent_height = match window.as_tiling_container() {
        Ok(tiling_window) => tiling_window
          .container_to_resize(false)?
          .and_then(|container| container.parent())
          .and_then(|parent| parent.height().ok())
          .map(|parent_height| {
            parent_height
              - tiling_window.inner_gap().to_pixels(monitor_rect.height())
                * tiling_window.tiling_siblings().count() as i32
          }),
        _ => window.parent().and_then(|parent| parent.height().ok()),
      };

      parent_height.map(|parent_height| {
        let delta_px = delta.to_pixels(parent_height);
        window_rect.height() + delta_px
      })
    }
    _ => None,
  };

  set_window_size(
    window.clone(),
    target_width
      .map(|target_width| LengthValue::new_px(target_width as f32)),
    target_height
      .map(|target_height| LengthValue::new_px(target_height as f32)),
    state,
  )?;

  Ok(())
}
