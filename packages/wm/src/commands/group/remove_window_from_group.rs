use anyhow::Context;

use crate::{
  commands::container::{move_container_within_tree, replace_container},
  models::{enclosing_group, GroupContainer, TilingWindow},
  traits::CommonGetters,
  wm_state::WmState,
};

/// Side of a group that a window is moved out to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EjectSide {
  /// Place the window directly before the group.
  Before,

  /// Place the window directly after the group.
  After,
}

/// Moves a window out of its group, placing it on the given side of the
/// group within the group's parent.
///
/// If the group only contains this window, the group is dissolved and
/// the window takes its place in the tree.
pub fn eject_window_from_group(
  window: &TilingWindow,
  side: EjectSide,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let group =
    enclosing_group(window).context("Window is not in a group.")?;
  let parent = group.parent().context("No parent.")?;

  if group.child_count() == 1 {
    dissolve_group(window, &group, state)?;
  } else {
    let group_index = group.index();

    let target_index = match side {
      EjectSide::Before => group_index,
      EjectSide::After => group_index + 1,
    };

    move_container_within_tree(
      &window.clone().into(),
      &parent,
      target_index,
      state,
    )?;
  }

  state
    .pending_sync
    .queue_container_to_redraw(window.clone())
    .queue_containers_to_redraw(parent.tiling_children());

  Ok(())
}

/// Replaces a group that only contains `window` with the window.
///
/// The window is removed from the group before the group is replaced,
/// since detaching a group that still has children would detach them.
fn dissolve_group(
  window: &TilingWindow,
  group: &GroupContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  let parent = group.parent().context("No parent.")?;
  let group_index = group.index();

  group
    .borrow_children_mut()
    .retain(|child| child.id() != window.id());

  group
    .borrow_child_focus_order_mut()
    .retain(|id| *id != window.id());

  *window.borrow_parent_mut() = None;

  // Dropping the tab strip destroys its window.
  let _ = group.take_tab_strip();

  replace_container(&window.clone().into(), &parent, group_index)?;

  state
    .pending_sync
    .queue_containers_to_redraw(parent.tiling_children());

  Ok(())
}
