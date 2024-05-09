use anyhow::Context;

use crate::containers::{
  traits::{CommonGetters, TilingDirectionGetters, TilingSizeGetters},
  SplitContainer,
};

/// Removes a split container from the tree and moves its children
/// into the parent container.
///
/// The children will be resized to fit the size of the split container.
pub fn flatten_split_container(
  split_container: SplitContainer,
) -> anyhow::Result<()> {
  let parent = split_container.parent().context("No parent.")?;

  let updated_children =
    split_container.children().into_iter().map(|child| {
      *child.borrow_parent_mut() = Some(parent.clone());

      // Resize tiling children to fit the size of the split container.
      if let Ok(tiling_child) = child.as_tiling_container() {
        tiling_child.set_tiling_size(
          split_container.tiling_size() * tiling_child.tiling_size(),
        );
      }

      // Inverse the tiling direction of any child split containers.
      if let Some(split_child) = child.as_split() {
        split_child.set_tiling_direction(
          split_container.tiling_direction().inverse(),
        );
      }

      child
    });

  let index = split_container.index();
  let focus_index = split_container.focus_index();

  // Insert child at its original index in the parent.
  for (child_index, child) in updated_children.enumerate() {
    parent
      .borrow_children_mut()
      .insert(index + child_index, child);
  }

  // Insert child at its original focus index in the parent.
  for (child_focus_index, child_id) in split_container
    .borrow_child_focus_order()
    .iter()
    .enumerate()
  {
    parent
      .borrow_child_focus_order_mut()
      .insert(focus_index + child_focus_index, *child_id);
  }

  // Remove the split container from the tree.
  parent
    .borrow_children_mut()
    .retain(|c| c.id() != split_container.id());

  parent
    .borrow_child_focus_order_mut()
    .retain(|id| *id != split_container.id());

  *split_container.borrow_parent_mut() = None;

  Ok(())
}
