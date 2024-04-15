use anyhow::bail;

use crate::containers::{
  traits::{CommonGetters, TilingSizeGetters},
  Container,
};

use super::resize_tiling_container;

/// Inserts a child container at the specified index.
///
/// The inserted child will be resized to fit the available space.
pub fn attach_container(
  child: &Container,
  target_parent: &Container,
  target_index: Option<usize>,
) -> anyhow::Result<()> {
  if !child.is_detached() {
    bail!("Cannot attach an already attached container.");
  }

  // Insert the child at the specified index.
  if let Some(target_index) = target_index {
    target_parent
      .borrow_children_mut()
      .insert(target_index, child.clone());
  } else {
    target_parent.borrow_children_mut().push_back(child.clone());
  }

  target_parent
    .borrow_child_focus_order_mut()
    .push_back(child.id());

  *child.borrow_parent_mut() = Some(target_parent.clone());

  // Resize the child and its siblings if it is a tiling container.
  if let Ok(child) = child.as_tiling_container() {
    let tiling_siblings = child.tiling_siblings().collect::<Vec<_>>();

    if tiling_siblings.is_empty() {
      child.set_tiling_size(1.0);
      return Ok(());
    }

    // Set initial tiling size to 0, and then size up the container
    // to the target size.
    let target_size = 1.0 / (tiling_siblings.len() + 1) as f32;
    child.set_tiling_size(0.0);
    resize_tiling_container(&child, target_size);
  }

  Ok(())
}
