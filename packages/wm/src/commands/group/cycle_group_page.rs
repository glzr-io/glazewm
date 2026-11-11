use anyhow::Context;

use crate::{
  commands::container::set_focused_descendant,
  models::{enclosing_group, TilingWindow},
  traits::CommonGetters,
  wm_state::WmState,
};

/// Focuses the next or previous page of the window's group.
///
/// Wraps around at either end. Does nothing if the window is not in a
/// group, or if the group has fewer than two pages.
pub fn cycle_group_page(
  window: &TilingWindow,
  is_next: bool,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let Some(group) = enclosing_group(window) else {
    return Ok(());
  };

  let page_count = group.child_count();
  if page_count < 2 {
    return Ok(());
  }

  let active_index = group.active_index();
  let target_index = if is_next {
    (active_index + 1) % page_count
  } else {
    (active_index + page_count - 1) % page_count
  };

  let target_page = group
    .children()
    .get(target_index)
    .cloned()
    .context("No page at target index.")?;

  set_focused_descendant(&target_page, None);

  let workspace = window.workspace().context("No workspace.")?;

  state
    .pending_sync
    .queue_focus_change()
    .queue_cursor_jump()
    .queue_container_to_redraw(group)
    .queue_workspace_to_reorder(workspace);

  Ok(())
}
