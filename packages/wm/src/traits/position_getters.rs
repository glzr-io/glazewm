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
        let inner_gap = match parent.tiling_direction() {
          TilingDirection::Vertical => vertical_gap,
          TilingDirection::Horizontal => horizontal_gap,
        };

        // Cache sibling count to avoid multiple iterations.
        #[allow(clippy::cast_possible_wrap)]
        let sibling_count = self.tiling_siblings().count() as i32;

        #[allow(
          clippy::cast_precision_loss,
          clippy::cast_possible_truncation,
          clippy::cast_possible_wrap
        )]
        let (width, height) = match parent.tiling_direction() {
          TilingDirection::Vertical => {
            let available_height = parent_rect.height()
              - inner_gap * sibling_count;

            let height =
              (self.tiling_size() * available_height as f32) as i32;

            (parent_rect.width(), height)
          }
          TilingDirection::Horizontal => {
            let available_width = parent_rect.width()
              - inner_gap * sibling_count;

            let width =
              (available_width as f32 * self.tiling_size()).round() as i32;

            (width, parent_rect.height())
          }
        };

        // Apply max_window_width constraint if configured.
        let (width, x_offset) = if let TilingDirection::Horizontal = parent.tiling_direction() {
          if let Some(workspace) = self.workspace() {
            if let Some(max_width) = workspace.max_window_width() {
              // Get the monitor to use for length calculations.
              let monitor = workspace.monitor();
              let scale_factor = monitor
                .and_then(|m| m.native().scale_factor().ok())
                .unwrap_or(1.);

              // Convert max_window_width to pixels.
              let max_width_px = max_width.to_px(parent_rect.width(), Some(scale_factor));

              // Calculate total width if all windows are at max width.
              // sibling_count is the count of OTHER siblings, so add 1 to include self.
              let window_count = sibling_count + 1;
              let total_gaps = inner_gap * sibling_count;  // Gaps between windows
              let total_max_width = max_width_px * window_count;

              // Only apply constraint if total max width plus gaps fits with room to spare.
              if total_max_width + total_gaps < parent_rect.width() {
                // Constrain this window's width to max_width_px.
                let constrained_width = width.min(max_width_px);
                
                // Calculate centering offset for the entire group.
                let total_constrained_width = total_max_width + total_gaps;
                let centering_offset = (parent_rect.width() - total_constrained_width) / 2;

                (constrained_width, centering_offset)
              } else {
                (width, 0)
              }
            } else {
              (width, 0)
            }
          } else {
            (width, 0)
          }
        } else {
          (width, 0)
        };

        let (x, y) = {
          let mut prev_siblings = self
            .prev_siblings()
            .filter_map(|sibling| sibling.as_tiling_container().ok());

          match prev_siblings.next() {
            None => (parent_rect.x() + x_offset, parent_rect.y()),
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
