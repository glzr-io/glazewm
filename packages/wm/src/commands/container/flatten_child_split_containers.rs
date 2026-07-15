use super::flatten_split_container;
use crate::{
  models::Container,
  traits::{CommonGetters, TilingDirectionGetters, TilingLayoutGetters},
};

/// Flattens any redundant split containers at the top-level of the given
/// parent container.
///
/// For example:
/// ```ignore,compile_fail
/// H[1 H[V[2, 3]]] -> H[1, 2, 3]
/// H[1 H[2, 3]] -> H[1, 2, 3]
/// H[V[1]] -> V[1]
/// ```
pub fn flatten_child_split_containers(
  parent: &Container,
) -> anyhow::Result<()> {
  if let Ok(parent) = parent.as_direction_container() {
    // Get children that are either tiling windows or split containers.
    let tiling_children = parent
      .children()
      .into_iter()
      .filter(|child| child.is_tiling_window() || child.is_split())
      .collect::<Vec<_>>();

    if tiling_children.len() == 1 {
      // Handle case where the parent is a split container and has a
      // single split container child.
      if let Some(split_child) = tiling_children[0].as_split() {
        let child_direction = split_child.tiling_direction();
        let child_layout = split_child.tiling_layout();
        flatten_split_container(split_child.clone())?;
        parent.set_tiling_direction(child_direction);
        parent.set_tiling_layout(child_layout);
      }
    } else {
      let split_children = tiling_children
        .into_iter()
        .filter_map(|child| child.as_split().cloned())
        .collect::<Vec<_>>();

      for split_child in split_children.iter().filter(|split_child| {
        split_child.tiling_direction() == parent.tiling_direction()
          && split_child.tiling_layout() == parent.tiling_layout()
      }) {
        // Additionally flatten redundant top-level split containers in
        // the child.
        if split_child.child_count() == 1 {
          if let Some(split_grandchild) =
            split_child.children()[0].as_split()
          {
            if split_grandchild.tiling_direction()
              == split_child.tiling_direction()
              && split_grandchild.tiling_layout()
                == split_child.tiling_layout()
            {
              flatten_split_container(split_grandchild.clone())?;
            }
          }
        }

        flatten_split_container(split_child.clone())?;
      }
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use wm_common::{TilingDirection, TilingLayout};

  use super::flatten_child_split_containers;
  use crate::{
    models::{SplitContainer, TilingWindow, Workspace},
    traits::{CommonGetters, TilingDirectionGetters, TilingLayoutGetters},
  };

  #[test]
  fn one_child_promotion_preserves_child_direction_and_layout() {
    let window = TilingWindow::mock().call();
    let split = SplitContainer::mock()
      .tiling_direction(TilingDirection::Vertical)
      .tiling_containers(vec![window.clone().into()])
      .call();
    split.set_tiling_layout(TilingLayout::Accordion);

    let workspace = Workspace::mock()
      .tiling_containers(vec![split.clone().into()])
      .call();

    flatten_child_split_containers(&workspace.clone().into())
      .expect("Single split child should flatten.");

    assert!(split.is_detached());
    assert_eq!(window.parent(), Some(workspace.clone().into()));
    assert_eq!(workspace.tiling_direction(), TilingDirection::Vertical);
    assert_eq!(workspace.tiling_layout(), TilingLayout::Accordion);
  }

  #[test]
  fn same_direction_with_different_layout_does_not_flatten() {
    let split = SplitContainer::mock()
      .tiling_containers(vec![TilingWindow::mock().call().into()])
      .call();
    split.set_tiling_layout(TilingLayout::Accordion);

    let workspace = Workspace::mock()
      .tiling_containers(vec![
        split.clone().into(),
        TilingWindow::mock().call().into(),
      ])
      .call();

    flatten_child_split_containers(&workspace.clone().into())
      .expect("Different layouts should remain nested.");

    assert_eq!(split.parent(), Some(workspace.into()));
  }
}
