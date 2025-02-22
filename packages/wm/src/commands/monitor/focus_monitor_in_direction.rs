use anyhow::Context;
use tracing::info;
use wm_common::Direction;

use crate::{
  commands::container::set_focused_descendant,
  traits::CommonGetters,
  wm_state::WmState,
};

pub fn focus_monitor_in_direction(
  direction: &Direction,
  state: &mut WmState,
) -> anyhow::Result<()> {
  info!("Focus_monitor_in_direction");
  let focused_workspace = state
    .focused_container()
    .and_then(|focused| focused.workspace())
    .context("No workspace is focused")?;

  let focused_monitor =
    focused_workspace.monitor().context("No focused monitor")?;

  let target_monitor = state
    .monitor_in_direction(&focused_monitor, direction)
    .context("No monitor")?;

  if let Some(target_monitor) = target_monitor {
    let target_workspace = target_monitor
      .displayed_workspace()
      .context("There is no workspace.")?;

    info!("Focusing Workspace: {target_workspace}");

    let container_to_focus = target_monitor
      .descendant_focus_order()
      .next()
      .unwrap_or_else(|| target_workspace.clone().as_container());

    set_focused_descendant(&container_to_focus, None);

    state
      .pending_sync
      .queue_focus_change()
      .queue_container_to_redraw(focused_workspace)
      .queue_container_to_redraw(target_workspace)
      .queue_cursor_jump();
  }
  Ok(())
}
