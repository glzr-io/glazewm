use anyhow::bail;

use super::resize_tiling_container;
use crate::{
  models::Container,
  traits::{CommonGetters, TilingSizeGetters},
};
use wm_common::TilingStrategy;

/// Inserts a child container at the specified index.
///
/// The inserted child will be resized to fit the available space.
pub fn attach_container(
  child: &Container,
  target_parent: &Container,
  target_index: Option<usize>,
  tiling_strategy: &TilingStrategy,
) -> anyhow::Result<()> {
  if !child.is_detached() {
    bail!("Cannot attach an already attached container.");
  }

  if let Some(target_index) = target_index {
    // Ensure target index is within the bounds of the parent's children.
    let target_index = target_index.clamp(0, target_parent.child_count());

    // Insert the child at the specified index.
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

    #[allow(clippy::cast_precision_loss)]
    match tiling_strategy {
      TilingStrategy::MasterStack => {
        let total = (tiling_siblings.len() + 1) as f32;

        if total == 2.0 {
          // Second window: each gets 50%.
          child.set_tiling_size(0.0);
          resize_tiling_container(&child, 0.5);
        } else {
          // 3+ windows: first child keeps 50%, the rest (including
          // the new one) share the other 50% equally.
          let stack_size = 0.5 / (total - 1.0);

          // Collect all tiling children and set sizes directly.
          let all_tiling: Vec<_> =
            target_parent.tiling_children().collect();

          for (i, tc) in all_tiling.iter().enumerate() {
            if i == 0 {
              tc.set_tiling_size(0.5);
            } else {
              tc.set_tiling_size(stack_size);
            }
          }
        }
      }
      TilingStrategy::Equal => {
        let target_size =
          1.0 / (tiling_siblings.len() + 1) as f32;
        child.set_tiling_size(0.0);
        resize_tiling_container(&child, target_size);
      }
    };
  }

  Ok(())
}
