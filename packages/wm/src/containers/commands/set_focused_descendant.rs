use crate::{
  containers::{traits::CommonBehavior, Container},
  wm_state::WmState,
};

pub fn set_focused_descendant(
  focused_descendant: Container,
  end_ancestor: Option<&Container>,
  state: &WmState,
) {
  let root = state.root_container.clone().into();
  let end_ancestor = end_ancestor.unwrap_or(&root);

  // Traverse upwards, setting the container as the last focused until the
  // root container or `end_ancestor` is reached.
  let mut target = focused_descendant;

  while let Some(parent) = target.parent() {
    if parent.id() == end_ancestor.id() {
      break;
    }

    {
      let mut child_focus_order = parent.borrow_child_focus_order_mut();

      if let Some(index) =
        child_focus_order.iter().position(|id| id == &target.id())
      {
        child_focus_order.remove(index);
        child_focus_order.push_front(target.id());
      }
    }

    target = parent.into();
  }
}
