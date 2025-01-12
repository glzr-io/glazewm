use crate::{
  models::RootContainer,
  traits::{CommonGetters, PositionGetters},
};

/// Sorts the root container's monitors from left-to-right and
/// top-to-bottom.
pub fn sort_monitors(root: RootContainer) -> anyhow::Result<()> {
  let monitors = root.monitors();

  // Create a tuple of monitors and their rects.
  let mut monitors_with_rect = monitors
    .into_iter()
    .map(|monitor| {
      let rect = monitor.to_rect()?.clone();
      anyhow::Ok((monitor, rect))
    })
    .try_collect::<Vec<_>>()?;

  // Sort monitors from left-to-right, top-to-bottom.
  monitors_with_rect.sort_by(|(_, rect_a), (_, rect_b)| {
    if rect_a.x() == rect_b.x() {
      rect_a.y().cmp(&rect_b.y())
    } else {
      rect_a.x().cmp(&rect_b.x())
    }
  });

  *root.borrow_children_mut() = monitors_with_rect
    .into_iter()
    .map(|(monitor, _)| monitor.into())
    .collect();

  Ok(())
}
