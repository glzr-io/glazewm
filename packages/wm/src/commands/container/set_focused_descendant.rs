use wm_common::VecDequeExt;

use crate::{models::Container, traits::CommonGetters};

/// Set a given container as the focused container up to and including the
/// end ancestor.
pub fn set_focused_descendant(
  focused_descendant: &Container,
  end_ancestor: Option<&Container>,
) {
  let mut target = focused_descendant.clone();

  // Traverse upwards, shifting the container's ancestors to the front in
  // their focus order.
  while let Some(parent) = target.parent() {
    parent
      .borrow_child_focus_order_mut()
      .shift_to_index(0, target.id());

    // Exit if we've reached the end ancestor.
    if end_ancestor
      .as_ref()
      .is_some_and(|end_ancestor| target.id() == end_ancestor.id())
    {
      break;
    }

    target = parent;
  }
}
