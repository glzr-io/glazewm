use std::collections::VecDeque;

use anyhow::Context;

use crate::containers::{
  traits::{CommonGetters, TilingSizeGetters},
  Container, SplitContainer, TilingContainer,
};

pub fn wrap_in_split_container(
  split_container: SplitContainer,
  target_parent: Container,
  target_children: Vec<TilingContainer>,
) -> anyhow::Result<()> {
  let starting_index = target_children
    .iter()
    .map(|child| child.index())
    .min()
    .context("Failed to get starting index.")?;

  target_parent
    .borrow_children_mut()
    .insert(starting_index, split_container.clone().into());

  let starting_focus_index = target_children
    .iter()
    .map(|child| child.focus_index())
    .min()
    .context("Failed to get starting focus index.")?;

  target_parent
    .borrow_child_focus_order_mut()
    .insert(starting_focus_index, split_container.id());

  // Get the total tiling size amongst all children.
  let total_tiling_size = target_children
    .iter()
    .map(|child| child.tiling_size())
    .sum::<f32>();

  let target_children_ids = target_children
    .iter()
    .map(|child| child.id())
    .collect::<Vec<_>>();

  let sorted_focus_ids = target_parent
    .borrow_child_focus_order()
    .iter()
    .filter(|id| target_children_ids.contains(id))
    .cloned()
    .collect::<VecDeque<_>>();

  // Set the split container's parent and tiling size.
  *split_container.borrow_parent_mut() = Some(target_parent.clone());
  split_container.set_tiling_size(total_tiling_size);

  // Move the children from their original parent to the split container.
  for target_child in target_children.iter() {
    *target_child.borrow_parent_mut() =
      Some(split_container.clone().into());

    split_container
      .borrow_children_mut()
      .push_back(target_child.clone().into());

    target_parent
      .borrow_children_mut()
      .retain(|child| child != &target_child.clone().into());

    target_parent
      .borrow_child_focus_order_mut()
      .retain(|id| id != &target_child.id());

    // Scale the tiling size to the new split container.
    target_child
      .set_tiling_size(target_child.tiling_size() / total_tiling_size);
  }

  // Add original focus order to split container.
  *split_container.borrow_child_focus_order_mut() = sorted_focus_ids;

  Ok(())
}
