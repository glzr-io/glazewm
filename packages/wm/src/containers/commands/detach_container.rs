use std::collections::VecDeque;

use anyhow::Context;

use crate::containers::{
  traits::{CommonGetters, TilingSizeGetters},
  Container,
};

use super::flatten_split_container;

/// Removes a container from the tree.
///
/// If the container is a tiling container, the siblings will be resized to
/// fill the freed up space. Will flatten empty parent split containers.
pub fn detach_container(child_to_remove: Container) -> anyhow::Result<()> {
  // Flatten the parent split container if it'll be empty after removing
  // the child.
  if let Some(split_parent) = child_to_remove
    .parent()
    .and_then(|parent| parent.as_split().cloned())
  {
    if split_parent.child_count() == 1 {
      flatten_split_container(split_parent)?;
    }
  }

  let parent = child_to_remove.parent().context("No parent.")?;

  parent
    .borrow_children_mut()
    .retain(|c| c.id() != child_to_remove.id());

  parent
    .borrow_child_focus_order_mut()
    .retain(|id| *id != child_to_remove.id());

  *child_to_remove.borrow_parent_mut() = None;
  *child_to_remove.borrow_children_mut() = VecDeque::new();

  // Resize the siblings if it is a tiling container.
  if let Ok(child_to_remove) = child_to_remove.as_tiling_container() {
    let tiling_siblings = parent.tiling_children().collect::<Vec<_>>();

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
