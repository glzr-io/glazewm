use anyhow::Context;
use wm_common::ParsedConfig;

use crate::{
  commands::container::{set_focused_descendant, wrap_in_container},
  models::{enclosing_group, GroupContainer, TilingWindow},
  traits::CommonGetters,
  wm_state::WmState,
};

/// Wraps a window in a new group at the window's position.
///
/// The window becomes the group's only page and is focused. Does nothing
/// if the window is already a page of a group.
pub fn create_group(
  window: &TilingWindow,
  config: &ParsedConfig,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if enclosing_group(window).is_some() {
    return Ok(());
  }

  let parent = window.parent().context("No parent.")?;
  let workspace = window.workspace().context("No workspace.")?;

  let group =
    GroupContainer::new(config.gaps.clone(), config.groups.clone());

  wrap_in_container(&group, &parent, &[window.clone().into()])?;
  set_focused_descendant(&window.clone().into(), None);

  state
    .pending_sync
    .queue_focus_change()
    .queue_containers_to_redraw(parent.tiling_children())
    .queue_workspace_to_reorder(workspace);

  Ok(())
}
