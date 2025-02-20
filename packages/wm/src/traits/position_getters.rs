use ambassador::delegatable_trait;
use wm_common::Rect;

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
      fn to_rect(&self) -> anyhow::Result<Rect> {
        let parent = self
          .parent()
          .and_then(|parent| parent.as_direction_container().ok())
          .context("Parent does not have a tiling direction.")?;
        let parent_rect = parent.to_rect()?;
        let (horizontal_gap, vertical_gap) = self.inner_gaps()?;

        match parent.tiling_direction() {
          TilingDirection::Accordion => {
            // For accordion, we stack windows with fixed 100px offset
            let base_width = parent_rect.width();
            let base_height = parent_rect.height();

            // Calculate offset based on position in stack, convert index
            // to i32
            let offset = (self.index() as i32) * 100;

            Ok(Rect::from_xy(
              parent_rect.x(),
              parent_rect.y() + offset,
              base_width,
              base_height,
            ))
          }
          TilingDirection::Vertical => {
            let inner_gap = vertical_gap;
            let available_height = parent_rect.height()
              - inner_gap * self.tiling_siblings().count() as i32;

            let height =
              (self.tiling_size() * available_height as f32) as i32;

            let (x, y) = {
              let mut prev_siblings = self
                .prev_siblings()
                .filter_map(|sibling| sibling.as_tiling_container().ok());

              match prev_siblings.next() {
                None => (parent_rect.x(), parent_rect.y()),
                Some(sibling) => {
                  let sibling_rect = sibling.to_rect()?;
                  (
                    parent_rect.x(),
                    sibling_rect.y() + sibling_rect.height() + inner_gap,
                  )
                }
              }
            };

            Ok(Rect::from_xy(x, y, parent_rect.width(), height))
          }
          TilingDirection::Horizontal => {
            let inner_gap = horizontal_gap;
            let available_width = parent_rect.width()
              - inner_gap * self.tiling_siblings().count() as i32;

            let width =
              (available_width as f32 * self.tiling_size()).round() as i32;

            let (x, y) = {
              let mut prev_siblings = self
                .prev_siblings()
                .filter_map(|sibling| sibling.as_tiling_container().ok());

              match prev_siblings.next() {
                None => (parent_rect.x(), parent_rect.y()),
                Some(sibling) => {
                  let sibling_rect = sibling.to_rect()?;
                  (
                    sibling_rect.x() + sibling_rect.width() + inner_gap,
                    parent_rect.y(),
                  )
                }
              }
            };

            Ok(Rect::from_xy(x, y, width, parent_rect.height()))
          }
        }
      }
    }
  };
}
