use crate::{
  containers::{traits::CommonGetters, Container},
  wm_state::WmState,
};

pub fn set_focused_descendant(
  focused_descendant: Container,
  end_ancestor: Option<Container>,
  state: &WmState,
) {
  let end_ancestor =
    end_ancestor.unwrap_or_else(|| state.root_container.clone().into());

  // Traverse upwards, setting the container as the last focused up to and
  // including the end ancestor.
  let mut target = focused_descendant;

  while let Some(parent) = target.parent() {
    {
      let mut child_focus_order = parent.borrow_child_focus_order_mut();

      if let Some(index) =
        child_focus_order.iter().position(|id| id == &target.id())
      {
        child_focus_order.remove(index);
        child_focus_order.push_front(target.id());
      }
    }

    // Exit if we've reached the end ancestor.
    if target.id() == end_ancestor.id() {
      break;
    }

    target = parent.into();
  }
}
