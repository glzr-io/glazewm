use anyhow::{bail, Context};
use wm_common::VecDequeExt;

use super::{attach_container, detach_container, resize_tiling_container};
use crate::{
  models::Container,
  traits::{CommonGetters, TilingSizeGetters},
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

  let focus_index = container_to_replace.focus_index();
  let tiling_size = container_to_replace
    .as_tiling_container()
    .map(|c| c.tiling_size());

  // TODO: This will cause issues if the detach causes a wrapping split
  // container to flatten. Currently, that scenario shouldn't be possible.
  // We also can't attach first before detaching, because detaching
  // removes child based on ID and both containers might have the same ID.
  detach_container(container_to_replace)?;

  attach_container(
    &replacement_container,
    &target_parent,
    Some(target_index),
  )?;

  // Shift to the correct focus index.
  target_parent
    .borrow_child_focus_order_mut()
    .shift_to_index(focus_index, replacement_container.id());

  // Match the tiling size of the replaced container if the replacement
  // is also a tiling container.
  if let Ok(tiling_size) = tiling_size {
    if let Ok(replacement_container) =
      replacement_container.as_tiling_container()
    {
      resize_tiling_container(&replacement_container, tiling_size);
    }
  }

  Ok(())
}
