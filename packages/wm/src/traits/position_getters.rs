use ambassador::delegatable_trait;
use anyhow::Context;
use wm_common::TilingDirection;
use wm_platform::Rect;

#[delegatable_trait]
pub trait PositionGetters {
  fn to_rect(&self) -> anyhow::Result<Rect>;
}

/// Gets the rect of a child in a container with the accordion layout.
///
/// Every child takes up the full parent rect, with the focused child
/// inset by `padding` on both ends so that its neighbors peek out. The
/// child before the focused one peeks out at the start, and the child
/// after it peeks out at the end.
///
/// The padding is clamped to a quarter of the parent's length so that
/// every child keeps at least half of the parent's length.
pub fn accordion_rect(
  parent_rect: &Rect,
  tiling_direction: &TilingDirection,
  padding: i32,
  index: Option<usize>,
  count: usize,
  focused_index: Option<usize>,
) -> anyhow::Result<Rect> {
  let index = index.context("Container not found in parent.")?;
  let focused_index = focused_index.unwrap_or(0);
  let last_index = count.saturating_sub(1);

  let parent_length = match tiling_direction {
    TilingDirection::Horizontal => parent_rect.width(),
    TilingDirection::Vertical => parent_rect.height(),
  };
  let padding = padding.clamp(0, parent_length.max(0) / 4);

  // Padding at the start and end of the child along the tiling direction.
  let (start, end) = if count <= 1 {
    (0, 0)
  } else if index == 0 {
    (0, padding)
  } else if index == last_index {
    (padding, 0)
  } else if index + 1 == focused_index {
    (0, padding * 2)
  } else if index == focused_index + 1 {
    (padding * 2, 0)
  } else {
    (padding, padding)
  };

  Ok(match tiling_direction {
    TilingDirection::Horizontal => Rect::from_xy(
      parent_rect.x() + start,
      parent_rect.y(),
      parent_rect.width() - start - end,
      parent_rect.height(),
    ),
    TilingDirection::Vertical => Rect::from_xy(
      parent_rect.x(),
      parent_rect.y() + start,
      parent_rect.width(),
      parent_rect.height() - start - end,
    ),
  })
}

/// Implements the `PositionGetters` trait for tiling containers that can
/// be resized. This is used by `SplitContainer` and `TilingWindow`.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with an `id` and a `parent` field.
#[macro_export]
macro_rules! impl_position_getters_as_resizable {
  ($struct_name:ident) => {
    impl PositionGetters for $struct_name {
      fn to_rect(&self) -> anyhow::Result<Rect> {
        let parent = self
          .parent()
          .and_then(|parent| parent.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        let parent_rect = parent.to_rect()?;

        if parent.tiling_layout() == wm_common::TilingLayout::Accordion {
          return $crate::traits::accordion_rect(
            &parent_rect,
            &parent.tiling_direction(),
            self.accordion_padding(match parent.tiling_direction() {
              TilingDirection::Vertical => parent_rect.height(),
              TilingDirection::Horizontal => parent_rect.width(),
            })?,
            parent.tiling_children().position(|c| c.id() == self.id()),
            parent.tiling_children().count(),
            parent
              .child_focus_order()
              .find_map(|c| c.as_tiling_container().ok())
              .and_then(|focused| {
                parent
                  .tiling_children()
                  .position(|c| c.id() == focused.id())
              }),
          );
        }

        let (horizontal_gap, vertical_gap) = self.inner_gaps()?;
        let inner_gap = match parent.tiling_direction() {
          TilingDirection::Vertical => vertical_gap,
          TilingDirection::Horizontal => horizontal_gap,
        };

        #[allow(
          clippy::cast_precision_loss,
          clippy::cast_possible_truncation,
          clippy::cast_possible_wrap
        )]
        let (width, height) = match parent.tiling_direction() {
          TilingDirection::Vertical => {
            let available_height = parent_rect.height()
              - inner_gap * self.tiling_siblings().count() as i32;

            let height =
              (self.tiling_size() * available_height as f32) as i32;

            (parent_rect.width(), height)
          }
          TilingDirection::Horizontal => {
            let available_width = parent_rect.width()
              - inner_gap * self.tiling_siblings().count() as i32;

            let width =
              (available_width as f32 * self.tiling_size()).round() as i32;

            (width, parent_rect.height())
          }
        };

        let (x, y) = {
          let mut prev_siblings = self
            .prev_siblings()
            .filter_map(|sibling| sibling.as_tiling_container().ok());

          match prev_siblings.next() {
            None => (parent_rect.x(), parent_rect.y()),
            Some(sibling) => {
              let sibling_rect = sibling.to_rect()?;

              match parent.tiling_direction() {
                TilingDirection::Vertical => (
                  parent_rect.x(),
                  sibling_rect.y() + sibling_rect.height() + inner_gap,
                ),
                TilingDirection::Horizontal => (
                  sibling_rect.x() + sibling_rect.width() + inner_gap,
                  parent_rect.y(),
                ),
              }
            }
          }
        };

        Ok(Rect::from_xy(x, y, width, height))
      }
    }
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  const PARENT_WIDTH: i32 = 1000;
  const PADDING: i32 = 30;

  /// Gets the x-position and width of a child in a horizontal accordion.
  fn horizontal_child(
    index: usize,
    count: usize,
    focused_index: usize,
  ) -> anyhow::Result<(i32, i32)> {
    let rect = accordion_rect(
      &Rect::from_xy(0, 0, PARENT_WIDTH, 500),
      &TilingDirection::Horizontal,
      PADDING,
      Some(index),
      count,
      Some(focused_index),
    )?;

    Ok((rect.x(), rect.width()))
  }

  #[test]
  fn single_child_fills_parent() -> anyhow::Result<()> {
    assert_eq!(horizontal_child(0, 1, 0)?, (0, PARENT_WIDTH));
    Ok(())
  }

  #[test]
  fn neighbors_peek_out_around_focused_child() -> anyhow::Result<()> {
    // Focused middle child is inset on both ends.
    assert_eq!(
      horizontal_child(1, 3, 1)?,
      (PADDING, PARENT_WIDTH - PADDING * 2)
    );

    // Previous child peeks out at the start, next child at the end.
    assert_eq!(horizontal_child(0, 3, 1)?, (0, PARENT_WIDTH - PADDING));
    assert_eq!(
      horizontal_child(2, 3, 1)?,
      (PADDING, PARENT_WIDTH - PADDING)
    );

    Ok(())
  }

  #[test]
  fn vertical_accordion_insets_height() -> anyhow::Result<()> {
    let rect = accordion_rect(
      &Rect::from_xy(0, 0, PARENT_WIDTH, 500),
      &TilingDirection::Vertical,
      PADDING,
      Some(0),
      2,
      Some(1),
    )?;

    assert_eq!(
      (rect.y(), rect.height(), rect.width()),
      (0, 500 - PADDING, PARENT_WIDTH)
    );

    Ok(())
  }

  #[test]
  fn padding_is_clamped_in_small_parents() -> anyhow::Result<()> {
    // Padding of 30px in a 100px parent is clamped to 25px, so the child
    // inset on one side by double padding keeps half the parent's width.
    let rect = accordion_rect(
      &Rect::from_xy(0, 0, 100, 500),
      &TilingDirection::Horizontal,
      PADDING,
      Some(0),
      3,
      Some(1),
    )?;
    assert_eq!((rect.x(), rect.width()), (0, 75));

    let rect = accordion_rect(
      &Rect::from_xy(0, 0, 100, 500),
      &TilingDirection::Horizontal,
      PADDING,
      Some(1),
      4,
      Some(2),
    )?;
    assert_eq!((rect.x(), rect.width()), (0, 50));

    Ok(())
  }

  #[test]
  fn missing_child_is_an_error() {
    assert!(accordion_rect(
      &Rect::from_xy(0, 0, PARENT_WIDTH, 500),
      &TilingDirection::Horizontal,
      PADDING,
      None,
      2,
      Some(0),
    )
    .is_err());
  }
}
