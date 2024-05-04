use anyhow::{bail, Context};

use crate::{
  common::VecDequeExt,
  containers::{
    traits::{CommonGetters, TilingSizeGetters},
    Container,
  },
};

/// Replaces a container at the specified index.
///
/// The replaced container will be detached from the tree.
pub fn replace_container(
  replacement_container: Container,
  target_parent: Container,
  target_index: usize,
) -> anyhow::Result<()> {
  if !replacement_container.is_detached() {
    bail!(
      "Cannot use an already attached container as replacement container."
    );
  }

  let container_to_replace = target_parent
    .children()
    .get(target_index)
    .cloned()
    .with_context(|| format!("No container at index {}.", target_index))?;

  if let (Ok(container_to_replace), Ok(replacement_container)) = (
    container_to_replace.as_tiling_container(),
    replacement_container.as_tiling_container(),
  ) {
    replacement_container
      .set_tiling_size(container_to_replace.tiling_size());
  }

  // Replace the container at the given index.
  target_parent
    .borrow_children_mut()
    .replace(&container_to_replace, replacement_container.clone());

  target_parent
    .borrow_child_focus_order_mut()
    .replace(&container_to_replace.id(), replacement_container.id());

  *replacement_container.borrow_parent_mut() = Some(target_parent.clone());
  *container_to_replace.borrow_parent_mut() = None;

  Ok(())
}
