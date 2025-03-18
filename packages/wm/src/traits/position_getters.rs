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
        let layout =
          self.workspace().expect("no workspace").tiling_layout();
        match layout {
          TilingLayout::Manual { tiling_direction } => {
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

                let width = (available_width as f32 * self.tiling_size())
                  .round() as i32;

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
          TilingLayout::MasterStack { master_window } => {
            let parent = self
              .parent()
              .and_then(|parent| parent.as_direction_container().ok())
              .context("Parent does not have a tiling direction.")?;
            let parent_rect = parent.to_rect()?;
            let (horizontal_gap, vertical_gap) = self.inner_gaps()?;

            // Get all siblings including self
            let all_containers: Vec<_> = parent
              .children()
              .into_iter()
              .filter_map(|child| child.as_tiling_container().ok())
              .collect();

            // If there's only one window, it takes up the whole space
            if all_containers.len() <= 1 {
              return Ok(parent_rect);
            }

            // Find position of this container in siblings
            let index = all_containers
              .iter()
              .position(|c| c.id() == self.id())
              .unwrap_or(0);

            // TODO - get from config
            let master_ratio = 0.5;

            println!("should be comparing..");
            let is_master = match master_window {
                Some(master) => {
                let master_id = master.id();
                let self_id = self.id();
                println!(
                  "{} Comparing master ID: {:?} with self ID: {:?}",
                  master_id == self_id,
                  master_id,
                  self_id
                );
                // println!("master {:#?} self {:#?}", master, self);
                master_id == self_id
              },
              None => {
                println!("master is none");
                false
              },
            };

            if is_master {
              println!("IS MASTER----------------------------------------------------");
              // Master container takes left side
              let master_width =
                (parent_rect.width() as f32 * master_ratio) as i32;
              Ok(Rect::from_xy(
                parent_rect.x(),
                parent_rect.y(),
                master_width,
                parent_rect.height(),
              ))
            } else {
              // Stack containers on right side
              let stack_count = all_containers.len(); // Exclude master
              let stack_position = index; // Position in stack (0-indexed)

              let master_width =
                (parent_rect.width() as f32 * master_ratio) as i32;
              let stack_width =
                parent_rect.width() - master_width - horizontal_gap;

              // Each stack container gets equal height
              let stack_height = (parent_rect.height()
                - vertical_gap * (stack_count - 1) as i32)
                / stack_count as i32;

              Ok(Rect::from_xy(
                parent_rect.x() + master_width + horizontal_gap,
                parent_rect.y()
                  + stack_position as i32 * (stack_height + vertical_gap),
                stack_width,
                stack_height,
              ))
            }
          }
        }
      }
    }
  };
}
