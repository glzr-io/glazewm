use std::collections::VecDeque;

use anyhow::Context;

use crate::{
  containers::{
    traits::{CommonGetters, DirectionGetters, TilingGetters},
    Container, SplitContainer,
  },
  wm_state::WmState,
};

pub fn flatten_split_container(
  split_container: SplitContainer,
  state: &WmState,
) -> anyhow::Result<()> {
  let parent = split_container.parent().context("No parent.")?;

  let updated_children = split_container.children().iter().map(|child| {
    *child.borrow_parent_mut() = Some(parent.clone());

    // Resize tiling children to fit the size of the split container.
    if let Ok(tiling_child) = child.as_tiling_container() {
      tiling_child.set_size_percent(
        split_container.size_percent() * tiling_child.size_percent(),
      );
    }

    // Inverse the tiling direction of any child split containers.
    if let Some(split_child) = child.as_split() {
      split_child.set_tiling_direction(
        split_container.tiling_direction().inverse(),
      );
    }
  });

  // let mut deque = VecDeque::from(vec![1, 2, 3, 4, 5]);
  // let position = 2;
  // let new_elements = vec![10, 11, 12];

  // deque.splice(position..position + 1, new_elements.iter().cloned());

  // println!("{:?}", deque);

  let focus_index = split_container.focus_index();
  parent
    .borrow_child_focus_order_mut()
    .iter_mut()
    .collect::<Vec<_>>()
    .splice(
      focus_index..focus_index,
      split_container
        .borrow_child_focus_order()
        .clone()
        .iter_mut(),
    );

  let index = split_container.index();
  parent
    .borrow_children_mut()
    .iter_mut()
    .collect::<Vec<_>>()
    .splice(index..index, updated_children.collect::<Vec<_>>());
  // .collect();
  // .retain(|id| id != &split_container.id());

  Ok(())
}
