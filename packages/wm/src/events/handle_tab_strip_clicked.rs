use anyhow::Context;
use wm_platform::TabStripId;

use crate::{
  commands::container::set_focused_descendant, traits::CommonGetters,
  wm_state::WmState,
};

/// Focuses the page at the given index of the group that owns the tab
/// strip.
///
/// Does nothing if no group owns the strip, or if the index is out of
/// bounds.
pub fn handle_tab_strip_clicked(
  strip_id: TabStripId,
  index: usize,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let group = state
    .root_container
    .descendants()
    .filter_map(|container| container.as_group().cloned())
    .find(|group| group.tab_strip_id() == Some(strip_id));

  let Some(group) = group else {
    return Ok(());
  };

  let Some(page) = group.children().get(index).cloned() else {
    return Ok(());
  };

  let workspace = group.workspace().context("No workspace.")?;

  set_focused_descendant(&page, None);

  state
    .pending_sync
    .queue_focus_change()
    .queue_container_to_redraw(group)
    .queue_workspace_to_reorder(workspace);

  Ok(())
}
