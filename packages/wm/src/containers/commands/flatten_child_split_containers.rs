use crate::containers::{
  traits::{CommonGetters, TilingDirectionGetters},
  Container,
};

use super::flatten_split_container;

/// Flattens any redundant split containers at the top-level of the given
/// parent container.
///
/// For example:
/// ```
/// H[1 H[V[2, 3]]] -> H[1, 2, 3]
/// H[V[1]] -> H[1]
/// ```
pub fn flatten_child_split_containers(
  parent: Container,
) -> anyhow::Result<()> {
  if let Ok(parent) = parent.as_direction_container() {
    // Get children that are either tiling windows or split containers.
    let tiling_children = parent
      .children()
      .into_iter()
      .filter(|child| child.is_tiling_window() || child.is_split())
      .collect::<Vec<_>>();

    match tiling_children.len() {
      1 => {
        // Handle case where the parent is a split container and has a single
        // split container child.
        if let Some(split_child) = tiling_children[0].as_split() {
          flatten_split_container(split_child.clone())?;
          parent.set_tiling_direction(parent.tiling_direction().inverse());
        }
      }
      _ => {
        for split_child in tiling_children
          .into_iter()
          .filter_map(|child| child.as_split().cloned())
          .filter(|split_child| split_child.child_count() == 1)
        {
          if let Some(split_grandchild) =
            split_child.children()[0].as_split()
          {
            flatten_split_container(split_grandchild.clone())?;
            flatten_split_container(split_child.clone())?;
          }
        }
      }
    }
  }

  Ok(())
}
