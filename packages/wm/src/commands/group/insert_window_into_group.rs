use anyhow::Context;

use crate::{
  commands::container::{
    move_container_within_tree, set_focused_descendant,
  },
  models::{GroupContainer, TilingWindow},
  traits::CommonGetters,
  wm_state::WmState,
};

/// Inserts a window as the page directly after the group's active page.
///
/// The window is then focused, which makes it the active page.
pub fn insert_window_into_group(
  window: &TilingWindow,
  group: &GroupContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let target_index = group.active_index() + 1;
  let workspace = window.workspace().context("No workspace.")?;

  move_container_within_tree(
    &window.clone().into(),
    &group.clone().into(),
    target_index,
    state,
  )?;

  set_focused_descendant(&window.clone().into(), None);

  state
    .pending_sync
    .queue_focus_change()
    .queue_container_to_redraw(group.clone())
    .queue_workspace_to_reorder(workspace);

  Ok(())
}
