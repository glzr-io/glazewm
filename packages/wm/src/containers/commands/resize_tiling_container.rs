use crate::containers::{
  traits::{CommonGetters, TilingGetters},
  TilingContainer,
};

const MIN_SIZE_PERCENTAGE: f32 = 0.01;

pub fn resize_tiling_container(
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
    available_size_percentage(&resizable_siblings);

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
    let sibling_resize_percentage = sibling_resize_percentage(
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

fn available_size_percentage(containers: &Vec<TilingContainer>) -> f32 {
  containers.iter().fold(0.0, |sum, container| {
    sum + container.size_percent() - MIN_SIZE_PERCENTAGE
  })
}

fn sibling_resize_percentage(
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
