use ambassador::delegatable_trait;
use wm_platform::Rect;

/// Calculates the start and length of a child on an accordion axis.
///
/// The inset rules mirror `AeroSpace`'s accordion layout: the first and
/// last children expose one edge, while the children adjacent to the most
/// recently focused child expose the opposite edge. Padding is clamped so
/// that applying two insets always leaves a positive child length.
pub(crate) fn accordion_axis_bounds(
  parent_start: i32,
  parent_length: i32,
  padding: i32,
  child_index: usize,
  child_count: usize,
  focused_index: usize,
) -> (i32, i32) {
  if child_count <= 1 {
    return (parent_start, parent_length);
  }

  let max_padding = parent_length.saturating_sub(1).max(0) / 2;
  let padding = padding.clamp(0, max_padding);

  let (leading, trailing) = if child_index == 0 {
    (0, padding)
  } else if child_index + 1 == child_count {
    (padding, 0)
  } else if child_index.checked_add(1) == Some(focused_index) {
    (0, padding * 2)
  } else if focused_index.checked_add(1) == Some(child_index) {
    (padding * 2, 0)
  } else {
    (padding, padding)
  };

  (parent_start + leading, parent_length - leading - trailing)
}

#[delegatable_trait]
pub trait PositionGetters {
  fn to_rect(&self) -> anyhow::Result<Rect>;
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
      #[allow(clippy::too_many_lines)]
      fn to_rect(&self) -> anyhow::Result<Rect> {
        let parent = self
          .parent()
          .and_then(|parent| parent.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;

        let parent_rect = parent.to_rect()?;

        if parent.tiling_layout() == wm_common::TilingLayout::Accordion {
          let tiling_children =
            parent.tiling_children().collect::<Vec<_>>();

          let child_index = tiling_children
            .iter()
            .position(|child| child.id() == self.id())
            .context("Container is not a tiling child of its parent.")?;

          let focused_child_id = parent
            .borrow_child_focus_order()
            .iter()
            .find(|child_id| {
              parent.child_by_id(child_id).is_some_and(|child| {
                child.is_tiling_window() || child.is_split()
              })
            })
            .copied();

          let focused_index = focused_child_id
            .and_then(|focused_id| {
              tiling_children
                .iter()
                .position(|child| child.id() == focused_id)
            })
            .unwrap_or(0);

          let child_count = tiling_children.len();

          return match parent.tiling_direction() {
            TilingDirection::Horizontal => {
              let padding = self.accordion_padding(parent_rect.width())?;
              let (x, width) = $crate::traits::accordion_axis_bounds(
                parent_rect.x(),
                parent_rect.width(),
                padding,
                child_index,
                child_count,
                focused_index,
              );

              Ok(Rect::from_xy(
                x,
                parent_rect.y(),
                width,
                parent_rect.height(),
              ))
            }
            TilingDirection::Vertical => {
              let padding =
                self.accordion_padding(parent_rect.height())?;
              let (y, height) = $crate::traits::accordion_axis_bounds(
                parent_rect.y(),
                parent_rect.height(),
                padding,
                child_index,
                child_count,
                focused_index,
              );

              Ok(Rect::from_xy(
                parent_rect.x(),
                y,
                parent_rect.width(),
                height,
              ))
            }
          };
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
  use std::collections::VecDeque;

  use wm_common::{TilingDirection, TilingLayout};

  use super::accordion_axis_bounds;
  use crate::{
    models::{Monitor, TilingWindow, Workspace},
    traits::{CommonGetters, PositionGetters, TilingLayoutGetters},
  };

  fn accordion_workspace(
    direction: TilingDirection,
  ) -> (Monitor, Vec<TilingWindow>) {
    let windows = (0..5)
      .map(|_| TilingWindow::mock().call())
      .collect::<Vec<_>>();

    let workspace = Workspace::mock()
      .tiling_direction(direction)
      .tiling_containers(windows.iter().cloned().map(Into::into).collect())
      .call();

    workspace.set_tiling_layout(TilingLayout::Accordion);
    *workspace.borrow_child_focus_order_mut() = VecDeque::from([
      windows[2].id(),
      windows[0].id(),
      windows[1].id(),
      windows[3].id(),
      windows[4].id(),
    ]);

    let monitor = Monitor::mock().workspaces(vec![workspace]).call();
    (monitor, windows)
  }

  #[test]
  fn accordion_single_child_uses_full_axis() {
    assert_eq!(accordion_axis_bounds(10, 500, 30, 0, 1, 0), (10, 500));
  }

  #[test]
  fn accordion_first_and_last_children_expose_outer_edges() {
    assert_eq!(accordion_axis_bounds(0, 500, 30, 0, 4, 1), (0, 470));
    assert_eq!(accordion_axis_bounds(0, 500, 30, 3, 4, 1), (30, 470));
  }

  #[test]
  fn accordion_children_adjacent_to_focus_expose_double_inset() {
    assert_eq!(accordion_axis_bounds(0, 500, 30, 1, 5, 2), (0, 440));
    assert_eq!(accordion_axis_bounds(0, 500, 30, 3, 5, 2), (60, 440));
    assert_eq!(accordion_axis_bounds(0, 500, 30, 2, 5, 2), (30, 440));
  }

  #[test]
  fn accordion_padding_is_clamped_to_positive_child_length() {
    assert_eq!(accordion_axis_bounds(5, 40, 100, 1, 3, 0), (43, 2));
    assert_eq!(accordion_axis_bounds(5, 40, -10, 1, 3, 0), (5, 40));
  }

  #[test]
  fn horizontal_accordion_positions_children_around_focus() {
    let (_monitor, windows) =
      accordion_workspace(TilingDirection::Horizontal);

    let rects = windows
      .iter()
      .map(|window| window.to_rect().expect("Window should have a rect."))
      .collect::<Vec<_>>();

    assert_eq!((rects[0].x(), rects[0].width()), (0, 1650));
    assert_eq!((rects[1].x(), rects[1].width()), (0, 1620));
    assert_eq!((rects[2].x(), rects[2].width()), (30, 1620));
    assert_eq!((rects[3].x(), rects[3].width()), (60, 1620));
    assert_eq!((rects[4].x(), rects[4].width()), (30, 1650));
  }

  #[test]
  fn vertical_accordion_positions_children_around_focus() {
    let (_monitor, windows) =
      accordion_workspace(TilingDirection::Vertical);

    let rects = windows
      .iter()
      .map(|window| window.to_rect().expect("Window should have a rect."))
      .collect::<Vec<_>>();

    assert_eq!((rects[0].y(), rects[0].height()), (0, 970));
    assert_eq!((rects[1].y(), rects[1].height()), (0, 940));
    assert_eq!((rects[2].y(), rects[2].height()), (30, 940));
    assert_eq!((rects[3].y(), rects[3].height()), (60, 940));
    assert_eq!((rects[4].y(), rects[4].height()), (30, 970));
  }
}
