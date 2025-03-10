use wm_common::{Direction, TilingDirection};

use super::set_focused_descendant;
use crate::{
  models::Container,
  traits::{CommonGetters, TilingDirectionGetters},
  wm_state::WmState,
};

pub fn focus_in_accordion(
  origin_container: &Container,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let origin_or_ancestor = origin_container.clone();

  // Check if we're in an accordion container
  if let Some(parent) = origin_or_ancestor.parent() {
    if let Ok(direction_parent) = parent.as_direction_container() {
      if matches!(
        direction_parent.tiling_direction(),
        TilingDirection::HorizontalAccordion
          | TilingDirection::VerticalAccordion
      ) {
        // For horizontal accordion, only respond to Up/Down
        // For vertical accordion, only respond to Left/Right
        let is_valid_direction = match direction_parent.tiling_direction()
        {
          TilingDirection::HorizontalAccordion => {
            matches!(direction, Direction::Up | Direction::Down)
          }
          TilingDirection::VerticalAccordion => {
            matches!(direction, Direction::Left | Direction::Right)
          }
          _ => false,
        };

        if is_valid_direction {
          // Get the next/prev sibling depending on the direction
          let focus_target = match direction {
            Direction::Up | Direction::Left => origin_or_ancestor
              .prev_siblings()
              .find_map(|c| c.as_tiling_container().ok()),
            _ => origin_or_ancestor
              .next_siblings()
              .find_map(|c| c.as_tiling_container().ok()),
          };

          if let Some(target) = focus_target {
            // Set focus to the target container
            set_focused_descendant(&target.into(), None);
            state.pending_sync.queue_focus_change().queue_cursor_jump();
            return Ok(());
          }
        }
      }
    }
  }

  // If not in an accordion or no target found, do nothing
  Ok(())
}
