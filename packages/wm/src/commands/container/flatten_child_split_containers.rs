use super::flatten_split_container;
use crate::{
  models::Container,
  traits::{CommonGetters, TilingDirectionGetters},
};

pub fn flatten_child_split_containers(
  parent: &Container,
) -> anyhow::Result<()> {
  if let Ok(parent) = parent.as_direction_container() {
    let tiling_children = parent
      .children()
      .into_iter()
      .filter(|child| child.is_tiling_window() || child.is_split())
      .collect::<Vec<_>>();

    if tiling_children.len() == 1 {
      if let Some(split_child) = tiling_children[0].as_split() {
        flatten_split_container(split_child.clone())?;
        parent.set_tiling_direction(parent.tiling_direction().inverse());
      }
    } else {
      let split_children = tiling_children
        .into_iter()
        .filter_map(|child| child.as_split().cloned())
        .collect::<Vec<_>>();

      for split_child in split_children.iter().filter(|split_child| {
        split_child.tiling_direction() == parent.tiling_direction()
      }) {
        if split_child.child_count() == 1 {
            // FIX: Check if children exist before accessing index 0
            if !split_child.children().is_empty() {
                if let Some(split_grandchild) = split_child.children()[0].as_split() {
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
