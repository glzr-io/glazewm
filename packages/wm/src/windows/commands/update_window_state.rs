use anyhow::Context;

use crate::{
  containers::{
    commands::{move_container_within_tree, replace_container},
    traits::CommonGetters,
    WindowContainer,
  },
  user_config::UserConfig,
  windows::{traits::WindowGetters, WindowState},
  wm_state::WmState,
};

pub fn update_window_state(
  window: WindowContainer,
  window_state: WindowState,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  if window.state() == window_state {
    return Ok(());
  }

  match window_state {
    WindowState::Tiling => set_tiling(window, state, config),
    _ => set_non_tiling(window, window_state, state),
  }
}

fn set_tiling(
  window: WindowContainer,
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  if let WindowContainer::NonTilingWindow(window) = window {
    let workspace = window
      .parent_workspace()
      .context("Window has no workspace.")?;

    // Get the position in the tree to insert the new tiling window. This
    // will be the window's previous tiling position if it has one, or
    // instead beside the last focused tiling window in the workspace.
    let (target_parent, target_index) = window
      .insertion_target()
      .or_else(|| {
        // Get the last focused tiling window within the workspace.
        let focused_window = window
          .descendant_focus_order()
          .filter(|c| c.is_tiling_window())
          .next()?;

        Some((focused_window.parent()?, focused_window.index() + 1))
      })
      .unwrap_or((workspace.into(), 0));

    let tiling_window =
      window.to_tiling(config.value.gaps.inner_gap.clone());

    // Replace the original window with the created tiling window.
    replace_container(
      tiling_window.clone().into(),
      window.parent().context("No parent")?,
      window.index(),
    )?;

    move_container_within_tree(
      tiling_window.clone().into(),
      target_parent,
      target_index,
    )?;

    state.add_container_to_redraw(tiling_window.into());
  }

  Ok(())
}

fn set_non_tiling(
  window: WindowContainer,
  window_state: WindowState,
  state: &mut WmState,
) -> anyhow::Result<()> {
  match window {
    WindowContainer::NonTilingWindow(window) => {
      window.set_state(window_state);
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

      let non_tiling_window = window.to_non_tiling(window_state);

      replace_container(
        non_tiling_window.clone().into(),
        window.parent().context("No parent")?,
        window.index(),
      )?;

      state.add_container_to_redraw(non_tiling_window.into());
    }
  }

  Ok(())
}
