use super::set_window_size;
use crate::{
  common::LengthValue,
  containers::{
    traits::{CommonGetters, PositionGetters, TilingSizeGetters},
    WindowContainer,
  },
  wm_state::WmState,
};

pub fn resize_window(
  window: WindowContainer,
  width_delta: Option<LengthValue>,
  height_delta: Option<LengthValue>,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let window_rect = window.to_rect()?;

  let target_width = match width_delta {
    Some(delta) => {
      let parent_width = match window.as_tiling_container() {
        Ok(tiling_window) => tiling_window
          .container_to_resize(true)?
          .and_then(|container| container.parent())
          .and_then(|parent| {
            parent.to_rect().ok().map(|rect| rect.width())
          })
          .and_then(|parent_width| {
            let (horizontal_gap, _) = tiling_window.inner_gaps().ok()?;

            Some(
              parent_width
                - horizontal_gap
                  * tiling_window.tiling_siblings().count() as i32,
            )
          }),
        _ => window.parent().and_then(|parent| {
          parent.to_rect().ok().map(|rect| rect.width())
        }),
      };

      parent_width.map(|parent_width| {
        window_rect.width() + delta.to_px(parent_width, None)
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
          .and_then(|parent| {
            parent.to_rect().ok().map(|rect| rect.width())
          })
          .and_then(|parent_height| {
            let (_, vertical_gap) = tiling_window.inner_gaps().ok()?;

            Some(
              parent_height
                - vertical_gap
                  * tiling_window.tiling_siblings().count() as i32,
            )
          }),
        _ => window.parent().and_then(|parent| {
          parent.to_rect().ok().map(|rect| rect.width())
        }),
      };

      parent_height.map(|parent_height| {
        window_rect.height() + delta.to_px(parent_height, None)
      })
    }
    _ => None,
  };

  set_window_size(
    window.clone(),
    target_width.map(LengthValue::from_px),
    target_height.map(LengthValue::from_px),
    state,
  )?;

  Ok(())
}
