use std::collections::VecDeque;

use anyhow::Context;

use crate::containers::{
  traits::{CommonGetters, TilingSizeGetters},
  Container,
};

use super::flatten_split_container;

/// Removes a container from the tree.
pub fn detach_container(child_to_remove: Container) -> anyhow::Result<()> {
  let containers_to_flatten =
    child_to_remove.containers_to_flatten_on_detach()?;

  // Get first ancestor that will remain after flattening.
  // TODO: Need to flatten parent *before* resizing siblings, but sibling +
  // sibling parent need to be flattened *after* resizing siblings.
  let adjusted_parent = child_to_remove
    .ancestors()
    .find(|ancestor| {
      containers_to_flatten
        .iter()
        .any(|to_flatten| to_flatten.id() != ancestor.id())
    })
    .context("No parent.")?;

  for split_container in containers_to_flatten {
    flatten_split_container(split_container)?;
  }

  adjusted_parent
    .borrow_children_mut()
    .retain(|c| c.id() != child_to_remove.id());

  adjusted_parent
    .borrow_child_focus_order_mut()
    .retain(|id| *id != child_to_remove.id());

  *child_to_remove.borrow_parent_mut() = None;
  *child_to_remove.borrow_children_mut() = VecDeque::new();

  // Resize the siblings if it is a tiling container.
  if let Ok(child_to_remove) = child_to_remove.as_tiling_container() {
    let tiling_siblings =
      adjusted_parent.tiling_children().collect::<Vec<_>>();

    let tiling_size_increment =
      child_to_remove.tiling_size() / tiling_siblings.len() as f32;

    // Adjust size of the siblings based on the freed up space.
    for sibling in &tiling_siblings {
      sibling
        .set_tiling_size(sibling.tiling_size() + tiling_size_increment);
    }
  }

  Ok(())
}
