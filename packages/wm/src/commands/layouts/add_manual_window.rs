use anyhow::Context;
use wm_common::WindowState;

use crate::{
  models::{Container, Workspace},
  traits::CommonGetters,
};

pub fn add_manual_window(
  target_parent: Option<Container>,
  window_state: &WindowState,
  target_container: Container,
  target_workspace: Workspace,
) -> anyhow::Result<(Container, usize)> {
  if let Some(target) = target_parent {
    return Ok((target.clone(), 0));
  }
  // For tiling windows, try to find a suitable tiling window to insert
  // next to.
  if *window_state == WindowState::Tiling {
    let sibling = match target_container {
      Container::TilingWindow(_) => Some(target_container),
      _ => target_workspace
        .descendant_focus_order()
        .find(Container::is_tiling_window),
    };

    if let Some(sibling) = sibling {
      return Ok((
        sibling.parent().context("No parent.")?,
        sibling.index() + 1,
      ));
    }
  }

  // Default to appending to workspace.
  Ok((
    target_workspace.clone().into(),
    target_workspace.child_count(),
  ))
}
