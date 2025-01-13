use anyhow::Context;
use wm_common::{LengthValue, Rect, WindowState};

use crate::{
  commands::container::resize_tiling_container,
  models::{NonTilingWindow, TilingWindow, WindowContainer},
  traits::{
    CommonGetters, PositionGetters, TilingSizeGetters, WindowGetters,
  },
  wm_state::WmState,
};

/// Arbitrary defaults for minimum floating window dimensions.
const MIN_FLOATING_WIDTH: i32 = 250;
const MIN_FLOATING_HEIGHT: i32 = 140;

pub fn set_window_size(
  window: WindowContainer,
  target_width: Option<LengthValue>,
  target_height: Option<LengthValue>,
  state: &mut WmState,
) -> anyhow::Result<()> {
  match window {
    WindowContainer::TilingWindow(window) => {
      set_tiling_window_size(&window, target_width, target_height, state)?;
    }
    WindowContainer::NonTilingWindow(window) => {
      if matches!(window.state(), WindowState::Floating(_)) {
        set_floating_window_size(
          window,
          target_width,
          target_height,
          state,
        )?;
      }
    }
  }

  Ok(())
}

fn set_tiling_window_size(
  window: &TilingWindow,
  target_width: Option<LengthValue>,
  target_height: Option<LengthValue>,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if let Some(target_width) = target_width {
    set_tiling_window_length(&window, &target_width, true, state)?;
  }

  if let Some(target_height) = target_height {
    set_tiling_window_length(&window, &target_height, false, state)?;
  }

  Ok(())
}

/// Updates either the width or height of a tiling window.
fn set_tiling_window_length(
  window: &TilingWindow,
  target_length: &LengthValue,
  is_width_resize: bool,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // When resizing a tiling window, the container to resize can actually be
  // an ancestor split container.
  let container_to_resize = window.container_to_resize(is_width_resize)?;

  if let Some(container_to_resize) = container_to_resize {
    let parent = container_to_resize.parent().context("No parent.")?;
    let (horizontal_gap, vertical_gap) =
      container_to_resize.inner_gaps()?;

    let parent_length = match is_width_resize {
      true => {
        parent.to_rect()?.width()
          - horizontal_gap * window.tiling_siblings().count() as i32
      }
      false => {
        parent.to_rect()?.height()
          - vertical_gap * window.tiling_siblings().count() as i32
      }
    };

    // Convert the target length to a tiling size.
    let tiling_size = target_length.to_percentage(parent_length);

    // Skip the resize if the window is already at the target size.
    if container_to_resize.tiling_size() - tiling_size != 0. {
      resize_tiling_container(&container_to_resize, tiling_size);

      state
        .pending_sync
        .containers_to_redraw
        .extend(parent.tiling_children().map(Into::into));
    }
  }

  Ok(())
}

fn set_floating_window_size(
  window: NonTilingWindow,
  target_width: Option<LengthValue>,
  target_height: Option<LengthValue>,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let monitor = window.monitor().context("No monitor")?;
  let monitor_rect = monitor.to_rect()?;
  let window_rect = window.to_rect()?;

  // Prevent resize from making the window smaller than minimum dimensions.
  // Always allow the size to be increased, even if the window would still
  // be within minimum dimension values.
  let length_with_clamp =
    |target_length: Option<i32>, current_length, min_length| {
      target_length.map_or(current_length, |target_length| {
        if target_length >= current_length {
          target_length
        } else {
          target_length.max(min_length)
        }
      })
    };

  let target_width_px = target_width
    .map(|target_width| target_width.to_px(monitor_rect.width(), None));

  let new_width = length_with_clamp(
    target_width_px,
    window_rect.width(),
    MIN_FLOATING_WIDTH,
  );

  let target_height_px = target_height
    .map(|target_height| target_height.to_px(monitor_rect.height(), None));

  let new_height = length_with_clamp(
    target_height_px,
    window_rect.height(),
    MIN_FLOATING_HEIGHT,
  );

  window.set_floating_placement(Rect::from_xy(
    window.floating_placement().x(),
    window.floating_placement().y(),
    new_width,
    new_height,
  ));

  state
    .pending_sync
    .containers_to_redraw
    .push(window.clone().into());

  Ok(())
}
