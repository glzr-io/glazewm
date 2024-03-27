use enum_dispatch::enum_dispatch;

use crate::containers::{Container, TilingContainer};

use super::CommonBehavior;

const MIN_SIZE_PERCENTAGE: f32 = 0.01;

#[enum_dispatch]
pub trait TilingBehavior: CommonBehavior {
  fn as_tiling_container(&self) -> TilingContainer;

  fn size_percent(&self) -> f32;

  fn set_size_percent(&self, size_percent: f32) -> ();

  fn insert_child(&self, target_index: usize, child: Container) {
    self
      .borrow_children_mut()
      .insert(target_index, child.clone());

    *child.borrow_parent_mut() = Some(self.as_tiling_container());

    if let Ok(child) = TryInto::<TilingContainer>::try_into(child) {
      let resizable_siblings = child.tiling_siblings().collect::<Vec<_>>();

      if resizable_siblings.is_empty() {
        child.set_size_percent(1.0);
        return;
      }

      // Set initial size percentage to 0, and then size up the container to `defaultPercent`.
      let default_percentage = 1.0 / (resizable_siblings.len() + 1) as f32;
      child.set_size_percent(0.0);
      resize_tiling_container(&child, default_percentage);
    }
  }
}

fn resize_tiling_container(
  container_to_resize: &TilingContainer,
  resize_percentage: f32,
) {
  let resizable_siblings =
    container_to_resize.tiling_siblings().collect::<Vec<_>>();

  // Ignore cases where the container to resize is a workspace or the only
  // child.
  if resizable_siblings.is_empty()
    || container_to_resize.as_workspace().is_some()
  {
    return;
  }

  // Get available size percentage amongst siblings.
  let available_size_percentage =
    get_available_size_percentage(&resizable_siblings);

  let min_resize_delta =
    MIN_SIZE_PERCENTAGE - container_to_resize.size_percent();

  // Prevent the container from being smaller than the minimum and larger
  // than space available from sibling containers.
  let clamped_resize_percentage = resize_percentage
    .max(min_resize_delta)
    .min(available_size_percentage);

  // Resize the container.
  container_to_resize.set_size_percent(
    container_to_resize.size_percent() + clamped_resize_percentage,
  );

  // Distribute the size percentage amongst its siblings.
  let sibling_count = resizable_siblings.len();
  for sibling in resizable_siblings {
    let sibling_resize_percentage = get_sibling_resize_percentage(
      &sibling,
      sibling_count,
      clamped_resize_percentage,
      available_size_percentage,
    );

    sibling.set_size_percent(
      sibling.size_percent() - sibling_resize_percentage,
    );
  }
}

fn get_available_size_percentage(
  containers: &Vec<TilingContainer>,
) -> f32 {
  containers.iter().fold(0.0, |sum, container| {
    sum + container.size_percent() - MIN_SIZE_PERCENTAGE
  })
}

fn get_sibling_resize_percentage(
  sibling_to_resize: &TilingContainer,
  sibling_count: usize,
  size_percentage: f32,
  available_size_percentage: f32,
) -> f32 {
  let con_available_size_percentage =
    sibling_to_resize.size_percent() - MIN_SIZE_PERCENTAGE;

  // Get percentage of resize that affects this container. The available
  // size percentage here can be 0 when the main container to resize is
  // shrunk from max size percentage.
  let resize_factor =
    if available_size_percentage == 0.0 || size_percentage < 0.0 {
      1.0 / sibling_count as f32
    } else {
      con_available_size_percentage / available_size_percentage
    };

  resize_factor * size_percentage
}

/// Implements the `TilingBehavior` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with a `children` field.
#[macro_export]
macro_rules! impl_tiling_behavior {
  ($struct_name:ident) => {
    impl TilingBehavior for $struct_name {
      fn as_tiling_container(&self) -> TilingContainer {
        self.clone().into()
      }

      fn size_percent(&self) -> f32 {
        self.0.borrow().size_percent
      }

      fn set_size_percent(&self, size_percent: f32) -> () {
        self.0.borrow_mut().size_percent = size_percent;
      }
    }
  };
}
