use crate::containers::{traits::CommonGetters, Container};

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
  match parent.child_count() {
    1 => {
      // Handle case where the parent is a split container and has a single
      // split container child.
      if parent.as_direction_container().is_ok() {
        if let Some(split_child) = parent.children()[0].as_split() {
          flatten_split_container(split_child.clone())?;
        }
      }
    }
    _ => {
      for split_child in parent
        .children()
        .iter()
        .filter_map(|child| child.as_split())
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

  Ok(())
}
