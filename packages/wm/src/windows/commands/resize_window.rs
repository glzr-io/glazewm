use anyhow::Context;

use crate::{
  common::{LengthValue, Rect, ResizeDimension, TilingDirection},
  containers::{
    commands::resize_tiling_container,
    traits::{CommonGetters, PositionGetters, TilingDirectionGetters},
    Container, WindowContainer,
  },
  user_config::UserConfig,
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
  config: &UserConfig,
) -> anyhow::Result<()> {
  match window {
    WindowContainer::TilingWindow(window) => resize_tiling_window(
      window,
      resize_dimension,
      resize_amount,
      state,
      config,
    ),
    WindowContainer::NonTilingWindow(window) => {
      if matches!(window.state(), WindowState::Floating(_)) {
        resize_floating_window(
          window,
          resize_dimension,
          resize_amount,
          state,
          config,
        )?;
      }

      Ok(())
    }
  }
}

fn resize_tiling_window(
  window: TilingWindow,
  resize_dimension: ResizeDimension,
  resize_amount: LengthValue,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  // When resizing a tiling window, the container to resize can actually be
  // an ancestor split container.
  if let Some(container_to_resize) =
    container_to_resize(window.clone(), resize_dimension)?
  {
    // Convert to a percentage to increase/decrease the window size by.
    // let tiling_size_delta =
    //   resize_amount.to_percentage(container_to_resize.tiling_size());

    // resize_tiling_container(container_to_resize, tiling_size_delta);

    // // TODO: Return early if `clamped_resize_percentage` is 0 to avoid
    // // unnecessary redraws.
    // state.add_container_to_redraw(
    //   container_to_resize.parent().context("No parent.")?,
    // );
  }

  Ok(())
}

fn container_to_resize(
  window: TilingWindow,
  resize_dimension: ResizeDimension,
) -> anyhow::Result<Option<Container>> {
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
    true => parent.into(),
    false => {
      let grandparent = parent.parent().context("No grandparent.")?;

      // Resize grandparent in layouts like H[1 V[2 H[3]]], where container
      // 3 is resized horizontally.
      if window.tiling_siblings().count() == 0 && grandparent.is_split() {
        grandparent
      } else {
        window.into()
      }
    }
  };

  // Ignore cases where the container to resize is a workspace or the only
  // child.
  if container_to_resize.tiling_siblings().count() > 0
    || container_to_resize.is_workspace()
  {
    return Ok(None);
  }

  Ok(Some(container_to_resize))
}

fn resize_floating_window(
  window: NonTilingWindow,
  resize_dimension: ResizeDimension,
  resize_amount: LengthValue,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let monitor = window.monitor().context("No monitor")?;
  let amount = resize_amount.to_pixels(monitor.width()?);

  let mut width = window.floating_placement().width();
  let mut height = window.floating_placement().height();

  match resize_dimension {
    ResizeDimension::Width => width += amount,
    ResizeDimension::Height => height += amount,
  }

  // Prevent resize from making the window smaller than minimum dimensions.
  // Always allow the size to be increased, even if the window would still
  // be within minimum dimension values.
  if amount < 0
    && (width < MIN_FLOATING_WIDTH || height < MIN_FLOATING_HEIGHT)
  {
    return Ok(());
  }

  window.set_floating_placement(Rect::from_xy(
    window.floating_placement().x(),
    window.floating_placement().y(),
    width,
    height,
  ));

  state.add_container_to_redraw(window.clone().into());

  Ok(())
}
