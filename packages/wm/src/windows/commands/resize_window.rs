use anyhow::Context;

use crate::{
  common::{LengthValue, Rect, ResizeDimension, TilingDirection},
  containers::{
    commands::resize_tiling_container,
    traits::{
      CommonGetters, PositionGetters, TilingDirectionGetters,
      TilingSizeGetters,
    },
    Container, DirectionContainer, TilingContainer, WindowContainer,
  },
  windows::{
    traits::WindowGetters, NonTilingWindow, TilingWindow, WindowState,
  },
  wm_state::WmState,
};

const MIN_FLOATING_WIDTH: i32 = 250;
const MIN_FLOATING_HEIGHT: i32 = 140;

pub fn resize_window(
  window: WindowContainer,
  resize_dimension: ResizeDimension,
  resize_amount: LengthValue,
  state: &mut WmState,
) -> anyhow::Result<()> {
  match window {
    WindowContainer::TilingWindow(window) => {
      resize_tiling_window(
        window,
        resize_dimension,
        resize_amount,
        state,
      )?;
    }
    WindowContainer::NonTilingWindow(window) => {
      if matches!(window.state(), WindowState::Floating(_)) {
        resize_floating_window(
          window,
          resize_dimension,
          resize_amount,
          state,
        )?;
      }
    }
  }

  Ok(())
}

fn resize_tiling_window(
  window: TilingWindow,
  resize_dimension: ResizeDimension,
  resize_amount: LengthValue,
  state: &mut WmState,
) -> anyhow::Result<()> {
  // When resizing a tiling window, the container to resize can actually be
  // an ancestor split container.
  if let Some(container_to_resize) =
    container_to_resize(window.clone(), resize_dimension)?
  {
    // Convert to a percentage to increase/decrease the tiling size by.
    let parent = container_to_resize.parent().context("No parent.")?;
    let resize_delta = resize_amount.to_percent(parent.width()?);

    if resize_delta != 0. {
      resize_tiling_container(
        &container_to_resize,
        container_to_resize.tiling_size() + resize_delta,
      );

      state
        .containers_to_redraw
        .extend(parent.tiling_children().map(|c| c.into()));
    }
  }

  Ok(())
}

fn container_to_resize(
  window: TilingWindow,
  resize_dimension: ResizeDimension,
) -> anyhow::Result<Option<TilingContainer>> {
  let parent = window.direction_container().context("No parent.")?;
  let tiling_direction = parent.tiling_direction();

  // Whether the resize is in the inverse of its tiling direction.
  let is_inverse_resize = match tiling_direction {
    TilingDirection::Horizontal => {
      resize_dimension == ResizeDimension::Height
    }
    TilingDirection::Vertical => {
      resize_dimension == ResizeDimension::Width
    }
  };

  let container_to_resize = match is_inverse_resize {
    true => match parent {
      // Prevent workspaces from being resized.
      DirectionContainer::Split(parent) => Some(parent.into()),
      _ => None,
    },
    false => {
      let grandparent = parent.parent().context("No grandparent.")?;

      match window.tiling_siblings().count() > 0 {
        // Window can only be resized if it has siblings.
        true => Some(window.into()),
        // Resize grandparent in layouts like H[1 V[2 H[3]]], where
        // container 3 is resized horizontally.
        false => match grandparent {
          Container::Split(grandparent) => Some(grandparent.into()),
          _ => None,
        },
      }
    }
  };

  Ok(container_to_resize)
}

fn resize_floating_window(
  window: NonTilingWindow,
  resize_dimension: ResizeDimension,
  resize_amount: LengthValue,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let monitor = window.monitor().context("No monitor")?;
  let resize_pixels = resize_amount.to_pixels(monitor.width()?);

  let mut new_width = window.floating_placement().width();
  let mut new_height = window.floating_placement().height();

  match resize_dimension {
    ResizeDimension::Width => new_width += resize_pixels,
    ResizeDimension::Height => new_height += resize_pixels,
  }

  // Prevent resize from making the window smaller than minimum dimensions.
  // Always allow the size to be increased, even if the window would still
  // be within minimum dimension values.
  if resize_pixels < 0
    && (new_width < MIN_FLOATING_WIDTH || new_height < MIN_FLOATING_HEIGHT)
  {
    return Ok(());
  }

  window.set_floating_placement(Rect::from_xy(
    window.floating_placement().x(),
    window.floating_placement().y(),
    new_width,
    new_height,
  ));

  state.add_container_to_redraw(window.clone().into());

  Ok(())
}
