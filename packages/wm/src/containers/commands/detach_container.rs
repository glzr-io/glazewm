use anyhow::Context;

use crate::containers::{
  traits::{CommonGetters, TilingGetters},
  Container, TilingContainer,
};

use super::flatten_split_container;

/// Removes a container from the tree.
pub fn detach_container(child_to_remove: Container) -> anyhow::Result<()> {
  let mut parent = child_to_remove.parent().context("No parent.")?;

  // Flatten the parent split container if it'll be empty after removing
  // the child.
  if let Some(split_parent) = parent.as_split().cloned() {
    if split_parent.children().len() == 1 {
      parent = parent.parent().context("No parent.")?;
      flatten_split_container(split_parent)?;
    }
  }

  parent
    .borrow_children_mut()
    .retain(|c| c.id() != child_to_remove.id());

  parent
    .borrow_child_focus_order_mut()
    .retain(|id| *id != child_to_remove.id());

  *child_to_remove.borrow_parent_mut() = None;

  // Resize the siblings if it is a tiling container.
  if let Ok(child_to_remove) = child_to_remove.as_tiling_container() {
    resize_sibling_containers(child_to_remove, parent)?;
  }

  Ok(())
}

fn resize_sibling_containers(
  child_to_remove: TilingContainer,
  parent: TilingContainer,
) -> anyhow::Result<()> {
  let tiling_siblings = parent
    .children()
    .into_iter()
    .filter_map(|c| c.as_tiling_container().ok())
    .collect::<Vec<_>>();

  let size_percent_increment =
    child_to_remove.size_percent() / tiling_siblings.len() as f32;

  // Adjust size of the siblings of the removed container.
  for container_to_resize in &tiling_siblings {
    container_to_resize.set_size_percent(
      container_to_resize.size_percent() + size_percent_increment,
    );
  }

  // If there is exactly *one* sibling to the detached container, then flatten that
  // sibling if it's a split container. This is to handle layouts like H[1 V[2 H[3]]],
  // where container 2 gets detached.
  if tiling_siblings.len() == 1 {
    if let Some(split_sibling) = tiling_siblings[0].as_split().cloned() {
      let split_sibling_parent =
        split_sibling.parent().context("No parent.")?;

      flatten_split_container(split_sibling)?;

      // Additionally flatten parent to handle deeply nested layouts.
      if let Some(split_sibling_parent) =
        split_sibling_parent.as_split().cloned()
      {
        flatten_split_container(split_sibling_parent)?;
      }
    }
  }

  Ok(())
}
