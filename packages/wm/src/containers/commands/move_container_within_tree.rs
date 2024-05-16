use anyhow::Context;

use crate::{
  common::VecDequeExt,
  containers::{traits::CommonGetters, Container},
};

use super::{attach_container, detach_container, set_focused_descendant};

pub fn move_container_within_tree(
  container_to_move: Container,
  target_parent: Container,
  target_index: usize,
) -> anyhow::Result<()> {
  // Get lowest common ancestor (LCA) between `container_to_move` and
  // `target_parent`. This could be the `target_parent` itself.
  let lowest_common_ancestor =
    lowest_common_ancestor(&container_to_move, &target_parent)
      .context("No common ancestor between containers.")?;

  if container_to_move.parent().context("No parent.")? == target_parent {
    target_parent
      .borrow_children_mut()
      .shift_to_index(target_index, container_to_move);

    return Ok(());
  }

  // Handle case where target parent is the LCA. For example, when swapping
  // sibling containers or moving a container to a direct ancestor.
  if target_parent == lowest_common_ancestor {
    return move_to_lowest_common_ancestor(
      container_to_move,
      lowest_common_ancestor,
      target_index,
    );
  }

  // Get ancestor of `container_to_move` that is a direct child of the LCA.
  // This could be the `container_to_move` itself.
  let container_to_move_ancestor = container_to_move
    .self_and_ancestors()
    .find(|ancestor| {
      ancestor.parent() == Some(lowest_common_ancestor.clone())
    })
    .context("TODO.")?;

  // Likewise, get ancestor of `target_parent` that is a direct child of
  // the LCA.
  let target_parent_ancestor = target_parent
    .self_and_ancestors()
    .find(|ancestor| {
      ancestor.parent() == Some(lowest_common_ancestor.clone())
    })
    .context("TODO.")?;

  // Get whether the container is the focused descendant in its original
  // subtree.
  let is_focused_descendant = container_to_move
    == container_to_move_ancestor
    || container_to_move_ancestor
      .descendant_focus_order()
      .next()
      .context("TODO.")?
      .self_and_ancestors()
      .any(|ancestor| ancestor == container_to_move);

  // Get whether the ancestor of `container_to_move` appears before
  // `target_parent`'s ancestor in the child focus order of the LCA.
  let original_focus_index = container_to_move_ancestor.focus_index();
  let is_subtree_focused =
    original_focus_index < target_parent_ancestor.focus_index();

  detach_container(container_to_move.clone())?;
  attach_container(
    &container_to_move.clone(),
    &target_parent.clone(),
    Some(target_index),
  )?;

  // Set `container_to_move` as focused descendant within target subtree if
  // its original subtree had focus more recently (even if the container is
  // not the last focused within that subtree).
  if is_subtree_focused {
    set_focused_descendant(
      container_to_move,
      Some(target_parent_ancestor.clone()),
    );
  }

  // If the focused descendant is moved to the targets subtree, then the
  // target's ancestor should be placed before the original ancestor in
  // LCA's child focus order.
  if is_focused_descendant && is_subtree_focused {
    lowest_common_ancestor
      .borrow_child_focus_order_mut()
      .shift_to_index(original_focus_index, target_parent_ancestor.id());
  }

  Ok(())
}

fn move_to_lowest_common_ancestor(
  container_to_move: Container,
  lowest_common_ancestor: Container,
  target_index: usize,
) -> anyhow::Result<()> {
  // Keep reference to focus index of container's ancestor in LCA's child
  // focus order.
  let original_focus_index = container_to_move
    .self_and_ancestors()
    .find(|ancestor| {
      ancestor.parent() == Some(lowest_common_ancestor.clone())
    })
    .context("TODO.")?
    .focus_index();

  detach_container(container_to_move.clone())?;

  // Adjust for when target index changes on detach of container. For
  // example, when shifting a top-level container to the right in a
  // workspace.
  let adjusted_target_index =
    target_index.clamp(0, lowest_common_ancestor.child_count());

  attach_container(
    &container_to_move.clone(),
    &lowest_common_ancestor.clone(),
    Some(adjusted_target_index),
  )?;

  lowest_common_ancestor
    .borrow_child_focus_order_mut()
    .shift_to_index(original_focus_index, container_to_move.id());

  Ok(())
}

/// Gets the lowest container in the tree that has both `container_a` and
/// `container_b` as descendants.
pub fn lowest_common_ancestor(
  container_a: &Container,
  container_b: &Container,
) -> Option<Container> {
  let mut ancestor_a = Some(container_a.clone());

  // Traverse upwards from container A.
  while let Some(current_ancestor_a) = ancestor_a {
    let mut ancestor_b = Some(container_b.clone());

    // Traverse upwards from container B.
    while let Some(current_ancestor_b) = ancestor_b {
      if current_ancestor_a == current_ancestor_b {
        return Some(current_ancestor_a);
      }

      ancestor_b = current_ancestor_b.parent();
    }

    ancestor_a = current_ancestor_a.parent();
  }

  None
}
