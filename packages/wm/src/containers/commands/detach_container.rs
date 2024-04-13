use anyhow::Context;

use crate::containers::{traits::CommonGetters, Container};

/// Removes a container from the tree.
pub fn detach_container(child_to_remove: Container) -> anyhow::Result<()> {
  let parent = child_to_remove.parent().context("No parent.")?;
  let grandparent = parent.parent();
  let siblings = child_to_remove.siblings();

  parent
    .borrow_children_mut()
    .retain(|c| c.id() != child_to_remove.id());

  parent
    .borrow_child_focus_order_mut()
    .retain(|id| *id != child_to_remove.id());

  *child_to_remove.borrow_parent_mut() = None;

  // // Resize the siblings if it is a tiling container.
  // if let Ok(child_to_remove) = child_to_remove.as_tiling_container() {
  //   resize_detached_container(child_to_remove)?;
  // }

  let parent_siblings = parent.siblings();
  let is_empty_split_container =
    !parent.has_children() && parent.is_split();

  // Get the freed up space after container is detached.
  let available_size_percentage = if is_empty_split_container {
    parent
      .as_resizable()
      .map(|r| r.size_percentage())
      .unwrap_or(0.0)
  } else {
    child_to_remove
      .as_resizable()
      .map(|r| r.size_percentage())
      .unwrap_or(0.0)
  };

  // Resize children of grandparent if `child_to_remove`'s parent is also to be detached.
  let containers_to_resize = if is_empty_split_container {
    grandparent.unwrap().children_of_type::<dyn IResizable>()
  } else {
    parent.children_of_type::<dyn IResizable>()
  };

  // If the parent of the removed child is now an empty split container, detach the
  // split container as well.
  // TODO: Move out calls to `ContainersToRedraw.Add(...)`, since detaching might not
  // always require a redraw.
  if is_empty_split_container {
    self
      .container_service
      .containers_to_redraw
      .add(grandparent.as_ref().unwrap());
    grandparent.unwrap().remove_child(&parent);
  } else {
    self.container_service.containers_to_redraw.add(&parent);
  }

  if available_size_percentage != 0.0 {
    let size_percentage_increment =
      available_size_percentage / containers_to_resize.len() as f32;

    // Adjust `SizePercentage` of the siblings of the removed container.
    for container_to_resize in containers_to_resize {
      let resizable = container_to_resize.as_resizable_mut().unwrap();
      resizable.set_size_percentage(
        resizable.size_percentage() + size_percentage_increment,
      );
    }
  }

  let detached_siblings = if is_empty_split_container {
    parent_siblings
  } else {
    siblings
  };

  let detached_parent = if is_empty_split_container {
    grandparent.unwrap()
  } else {
    parent
  };

  // If there is exactly *one* sibling to the detached container, then flatten that
  // sibling if it's a split container. This is to handle layouts like H[1 V[2 H[3]]],
  // where container 2 gets detached.
  if detached_siblings.len() == 1
    && matches!(detached_siblings[0].as_ref(), Container::Split(_))
    && !matches!(child_to_remove.as_ref(), Container::Workspace(_))
  {
    self.bus.invoke(FlattenSplitContainerCommand {
      split_container: detached_siblings[0].as_split_container().unwrap(),
    });
    if !matches!(detached_parent.as_ref(), Container::Workspace(_)) {
      self.bus.invoke(FlattenSplitContainerCommand {
        split_container: detached_parent.as_split_container().unwrap(),
      });
    }
  }

  Ok(())
}

fn resize_detached_container(
  child_to_remove: Container,
) -> anyhow::Result<()> {
  Ok(())
}
