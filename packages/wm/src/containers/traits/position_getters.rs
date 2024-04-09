use enum_dispatch::enum_dispatch;

use crate::{
  common::Rect,
  containers::{
    Container, DirectionContainer, TilingContainer, WindowContainer,
  },
};

#[enum_dispatch]
pub trait PositionGetters {
  fn width(&self) -> anyhow::Result<i32>;

  fn height(&self) -> anyhow::Result<i32>;

  fn x(&self) -> anyhow::Result<i32>;

  fn y(&self) -> anyhow::Result<i32>;

  fn to_rect(&self) -> anyhow::Result<Rect> {
    Ok(Rect::from_xy(
      self.x()?,
      self.y()?,
      self.width()?,
      self.height()?,
    ))
  }
}

/// Implements the `PositionGetters` trait for tiling containers that can
/// be resized. Specifically, this is for `SplitContainer` and `TilingWindow`.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with an `id` and a `parent` field.
#[macro_export]
macro_rules! impl_position_getters_as_resizable {
  ($struct_name:ident) => {
    impl PositionGetters for $struct_name {
      fn width(&self) -> anyhow::Result<i32> {
        let parent = self
          .parent()
          .and_then(|p| p.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        match parent.tiling_direction() {
          TilingDirection::Vertical => parent.width(),
          TilingDirection::Horizontal => {
            let inner_gap = self.inner_gap().to_pixels(
              self
                .parent_monitor()
                .context("No parent monitor.")?
                .width()?,
            );

            let available_width = parent.width()?
              - inner_gap * self.tiling_siblings().count() as i32;

            Ok((self.size_percent() * available_width as f32) as i32)
          }
        }
      }

      fn height(&self) -> anyhow::Result<i32> {
        let parent = self
          .parent()
          .and_then(|p| p.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        match parent.tiling_direction() {
          TilingDirection::Horizontal => parent.height(),
          TilingDirection::Vertical => {
            let inner_gap = self.inner_gap().to_pixels(
              self
                .parent_monitor()
                .context("No parent monitor.")?
                .width()?,
            );

            let available_height = parent.height()?
              - inner_gap * self.tiling_siblings().count() as i32;

            Ok((self.size_percent() * available_height as f32) as i32)
          }
        }
      }

      fn x(&self) -> anyhow::Result<i32> {
        let parent = self
          .parent()
          .and_then(|p| p.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        let mut prev_siblings = self
          .prev_siblings()
          .filter_map(|c| c.as_tiling_container().ok());

        match prev_siblings.next() {
          None => parent.x(),
          Some(prev_sibling) => {
            if parent.tiling_direction() == TilingDirection::Vertical {
              return parent.x();
            }

            let inner_gap = self.inner_gap().to_pixels(
              self
                .parent_monitor()
                .context("No parent monitor.")?
                .width()?,
            );

            Ok(prev_sibling.x()? + prev_sibling.width()? + inner_gap)
          }
        }
      }

      fn y(&self) -> anyhow::Result<i32> {
        let parent = self
          .parent()
          .and_then(|p| p.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        let mut prev_siblings = self
          .prev_siblings()
          .filter_map(|c| c.as_tiling_container().ok());

        match prev_siblings.next() {
          None => parent.y(),
          Some(prev_sibling) => {
            if parent.tiling_direction() == TilingDirection::Horizontal {
              return parent.y();
            }

            let inner_gap = self.inner_gap().to_pixels(
              self
                .parent_monitor()
                .context("No parent monitor.")?
                .width()?,
            );

            Ok(prev_sibling.y()? + prev_sibling.height()? + inner_gap)
          }
        }
      }
    }
  };
}
