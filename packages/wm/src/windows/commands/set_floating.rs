use anyhow::Context;

use crate::{
  containers::{
    commands::{move_container_within_tree, replace_container},
    traits::CommonGetters,
    WindowContainer,
  },
  windows::{traits::WindowGetters, WindowState},
  wm_state::WmState,
};

pub fn set_floating(
  window: WindowContainer,
  state: &mut WmState,
) -> anyhow::Result<()> {
  if window.state() == WindowState::Floating {
    return Ok(());
  }

  match window {
    WindowContainer::NonTilingWindow(window) => {
      window.set_state(WindowState::Floating);
      state.add_container_to_redraw(window.into());
    }
    WindowContainer::TilingWindow(window) => {
      let workspace = window
        .parent_workspace()
        .context("Window has no workspace.")?;

      move_container_within_tree(
        window.clone().into(),
        workspace.clone().into(),
        workspace.child_count() - 1,
      )?;

      let floating_window = window.to_non_tiling(WindowState::Floating);

      replace_container(
        floating_window.clone().into(),
        window.parent().context("No parent")?,
        window.index(),
      )?;

      state.add_container_to_redraw(floating_window.into());
    }
  }

  Ok(())
}
