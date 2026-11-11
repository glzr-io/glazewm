use wm_platform::Direction;

use crate::{
  commands::{
    container::move_container_within_tree,
    group::{eject_window_from_group, EjectSide},
  },
  models::{GroupContainer, TilingWindow},
  traits::CommonGetters,
  wm_state::WmState,
};

/// Moves a page of a group in the given direction.
///
/// Left and right reorder the window amongst the group's pages. If there
/// is no page to move past, or the group only has one page, the window
/// is moved out of the group on that side instead.
///
/// Up and down always move the window out of the group, on the side of
/// the movement.
///
/// Returns `true` if the move is complete. Returns `false` if the window
/// was moved out of the group vertically, in which case the window still
/// needs to be moved by the regular tiling logic.
pub fn move_window_in_group(
  window: &TilingWindow,
  group: &GroupContainer,
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<bool> {
  let is_horizontal =
    matches!(direction, Direction::Left | Direction::Right);

  if is_horizontal && group.child_count() > 1 {
    let target_index = match direction {
      Direction::Left => window.index().checked_sub(1),
      _ => Some(window.index() + 1)
        .filter(|index| *index < group.child_count()),
    };

    if let Some(target_index) = target_index {
      move_container_within_tree(
        &window.clone().into(),
        &group.clone().into(),
        target_index,
        state,
      )?;

      state.pending_sync.queue_container_to_redraw(group.clone());
      return Ok(true);
    }
  }

  let side = match direction {
    Direction::Left | Direction::Up => EjectSide::Before,
    Direction::Right | Direction::Down => EjectSide::After,
  };

  eject_window_from_group(window, side, state)?;

  Ok(is_horizontal)
}
