use anyhow::bail;

use crate::containers::{
  traits::{CommonGetters, TilingGetters},
  Container, TilingContainer,
};

use super::resize_tiling_container;

/// Inserts a child container at the specified index. Only tiling
/// containers can have children.
///
/// The inserted child will be resized to fit the available space.
pub fn attach_container(
  child: Container,
  target_parent: &TilingContainer,
  target_index: usize,
) -> anyhow::Result<()> {
  if !child.is_detached() {
    bail!("Cannot attach an already attached container.");
  }

  target_parent
    .borrow_children_mut()
    .insert(target_index, child.clone());

  target_parent
    .borrow_child_focus_order_mut()
    .push_back(child.id());

  *child.borrow_parent_mut() = Some(target_parent.as_tiling_container()?);

  // Resize the child and its siblings if it is a tiling container.
  if let Ok(child) = child.as_tiling_container() {
    let resizable_siblings = child.tiling_siblings().collect::<Vec<_>>();

    if resizable_siblings.is_empty() {
      child.set_size_percent(1.0);
      return Ok(());
    }

    // Set initial size percentage to 0, and then size up the container
    // to the default percentage.
    let default_percentage = 1.0 / (resizable_siblings.len() + 1) as f32;
    child.set_size_percent(0.0);
    resize_tiling_container(&child, default_percentage);
  }

  Ok(())
}
