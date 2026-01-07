use crate::{
  models::RootContainer,
  traits::{CommonGetters, PositionGetters},
};

/// Sorts the root container's monitors according to configuration, falling
/// back to physical position (left-to-right, top-to-bottom) for
/// unconfigured monitors.
pub fn sort_monitors_with_config(
  root: &RootContainer,
  monitor_configs: &[wm_common::MonitorConfig],
) -> anyhow::Result<()> {
  let monitors = root.monitors();

  // Create a tuple of monitors and their rects.
  let mut monitors_with_rect = monitors
    .into_iter()
    .map(|monitor| {
      let rect = monitor.to_rect()?;
      anyhow::Ok((monitor, rect))
    })
    .try_collect::<Vec<_>>()?;

  // Sort monitors first by configured order, then by physical position
  monitors_with_rect.sort_by(
    |(monitor_a, rect_a), (monitor_b, rect_b)| {
      // Get configured position for monitor A (index in config list)
      let config_pos_a = monitor_configs.iter().position(|config| {
        monitor_a
          .native()
          .machine_id()
          .ok()
          .flatten()
          .is_some_and(|machine_id| machine_id == config.machine_id)
      });

      // Get configured position for monitor B (index in config list)
      let config_pos_b = monitor_configs.iter().position(|config| {
        monitor_b
          .native()
          .machine_id()
          .ok()
          .flatten()
          .is_some_and(|machine_id| machine_id == config.machine_id)
      });

      match (config_pos_a, config_pos_b) {
        // Both have configured positions - sort by position in config
        (Some(a), Some(b)) => a.cmp(&b),
        // Only A has a configured position - A comes first
        (Some(_), None) => std::cmp::Ordering::Less,
        // Only B has a configured position - B comes first
        (None, Some(_)) => std::cmp::Ordering::Greater,
        // Neither has a configured position - sort by physical position
        (None, None) => {
          if rect_a.x() == rect_b.x() {
            rect_a.y().cmp(&rect_b.y())
          } else {
            rect_a.x().cmp(&rect_b.x())
          }
        }
      }
    },
  );

  *root.borrow_children_mut() = monitors_with_rect
    .into_iter()
    .map(|(monitor, _)| monitor.into())
    .collect();

  Ok(())
}
