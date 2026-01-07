use super::flatten_split_container;
use crate::{
  models::{Container, SplitContainer},
  traits::{CommonGetters, TilingDirectionGetters},
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
      .filter(|child| {
        matches!(child, Container::TilingWindow(_) | Container::Split(_))
      })
      .collect::<Vec<_>>();

    if tiling_children.len() == 1 {
      // Handle case where the parent is a split container and has a
      // single split container child.
      if let Ok(split_child) =
        <&SplitContainer>::try_from(&tiling_children[0])
      {
        flatten_split_container(split_child.clone())?;
        parent.set_tiling_direction(parent.tiling_direction().inverse());
      }
    } else {
      let split_children = tiling_children
        .into_iter()
        .filter_map(|child| SplitContainer::try_from(child.clone()).ok())
        .collect::<Vec<_>>();

      for split_child in split_children.iter().filter(|split_child| {
        split_child.tiling_direction() == parent.tiling_direction()
      }) {
        // Additionally flatten redundant top-level split containers in
        // the child.
        if split_child.child_count() == 1 {
          if let Ok(split_grandchild) =
            <&SplitContainer>::try_from(&split_child.children()[0])
          {
            flatten_split_container(split_grandchild.clone())?;
          }
        }

        flatten_split_container(split_child.clone())?;
      }
    }
  }

  Ok(())
}
