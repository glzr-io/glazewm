use crate::{
  commands::group::{eject_window_from_group, EjectSide},
  models::{enclosing_group, TilingWindow},
  wm_state::WmState,
};

/// Removes a window from its group, placing it after the group.
///
/// If the group only contains this window, the group is dissolved.
/// Does nothing if the window is not in a group.
pub fn ungroup_window(
  window: &TilingWindow,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if enclosing_group(window).is_none() {
    return Ok(());
  }

  eject_window_from_group(window, EjectSide::After, state)
}
