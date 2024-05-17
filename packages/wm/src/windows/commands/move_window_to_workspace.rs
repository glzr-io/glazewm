use anyhow::Context;

use crate::{
  containers::{
    commands::move_container_within_tree, traits::CommonGetters,
    WindowContainer,
  },
  user_config::UserConfig,
  windows::TilingWindow,
  wm_state::WmState,
};

pub fn move_window_to_workspace(
  window: WindowContainer,
  workspace_name: &str,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  match window {
    WindowContainer::TilingWindow(window) => {
      move_tiling_window_to_workspace(
        window,
        workspace_name,
        state,
        config,
      )
    }
    WindowContainer::NonTilingWindow(non_tiling_window) => {
      todo!()
    }
  }
}

fn move_tiling_window_to_workspace(
  window_container_to_move: TilingWindow,
  workspace_name: &str,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspaces = state.workspaces();
  let workspace = workspaces
    .iter()
    .find(|workspace| workspace.config().name == workspace_name)
    .unwrap();

  let target_container =
    workspace.descendant_focus_order().last().unwrap();

  move_container_within_tree(
    window_container_to_move.clone().into(),
    target_container,
    workspace.index(),
    state,
  )?;

  Ok(())
}
