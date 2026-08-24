use anyhow::Context;
use wm_common::TilingStrategy;

use super::flatten_split_container;
use crate::{
  models::Container,
  traits::{CommonGetters, TilingSizeGetters, MIN_TILING_SIZE},
};

/// Removes a container from the tree.
///
/// If the container is a tiling container, the siblings will be resized to
/// fill the freed up space. Will flatten empty parent split containers.
#[allow(clippy::needless_pass_by_value)]
pub fn detach_container(
  child_to_remove: Container,
  tiling_strategy: &TilingStrategy,
) -> anyhow::Result<()> {
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

  // Resize the siblings if it is a tiling container.
  if let Ok(child_to_remove) = child_to_remove.as_tiling_container() {
    let tiling_siblings = parent.tiling_children().collect::<Vec<_>>();

    #[allow(clippy::cast_precision_loss)]
    match tiling_strategy {
      TilingStrategy::MasterStack => {
        let total = tiling_siblings.len() as f32;

        if total == 1.0 {
          tiling_siblings[0].set_tiling_size(1.0);
        } else if total >= 2.0 {
          // Re-apply master-stack sizing: first child keeps 50%,
          // the rest share the other 50% equally.
          let stack_size = 0.5 / (total - 1.0);

          for (i, sibling) in tiling_siblings.iter().enumerate() {
            if i == 0 {
              sibling.set_tiling_size(0.5);
            } else {
              sibling.set_tiling_size(stack_size);
            }
          }
        }
      }
      TilingStrategy::Equal => {
        // TODO: Share logic with `resize_tiling_container`.
        let available_size =
          tiling_siblings.iter().fold(0.0, |sum, container| {
            sum + container.tiling_size() - MIN_TILING_SIZE
          });

        // Adjust size of the siblings based on the freed up space.
        for sibling in &tiling_siblings {
          let resize_factor =
            (sibling.tiling_size() - MIN_TILING_SIZE) / available_size;

          let size_delta =
            resize_factor * child_to_remove.tiling_size();
          sibling.set_tiling_size(sibling.tiling_size() + size_delta);
        }
      }
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::TilingStrategy;

  use super::detach_container;
  use crate::{
    commands::container::attach_container,
    models::{TilingWindow, Workspace},
    traits::{CommonGetters, TilingSizeGetters},
  };

  #[test]
  fn masterstack_close_returns_to_even_split() {
    let strategy = TilingStrategy::MasterStack;
    let workspace = Workspace::mock().call();

    // Insert window A (100%).
    let a = TilingWindow::mock().call();
    attach_container(
      &a.clone().into(),
      &workspace.clone().into(),
      None,
      &strategy,
    )
    .unwrap();

    // Insert window B (50% | 50%).
    let b = TilingWindow::mock().call();
    attach_container(
      &b.clone().into(),
      &workspace.clone().into(),
      None,
      &strategy,
    )
    .unwrap();

    // Insert window C (50% | 25% | 25%).
    let c = TilingWindow::mock().call();
    attach_container(
      &c.clone().into(),
      &workspace.clone().into(),
      None,
      &strategy,
    )
    .unwrap();

    let sizes: Vec<f32> = workspace
      .tiling_children()
      .map(|child| child.tiling_size())
      .collect();
    assert_eq!(sizes, vec![0.5, 0.25, 0.25], "after inserting 3rd window");

    // Close window C -> should return to 50% | 50%.
    detach_container(c.into(), &strategy).unwrap();

    let sizes: Vec<f32> = workspace
      .tiling_children()
      .map(|child| child.tiling_size())
      .collect();
    assert_eq!(sizes, vec![0.5, 0.5], "after closing 3rd window");
  }
}
