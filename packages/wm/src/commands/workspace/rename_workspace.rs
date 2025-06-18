use anyhow::{bail, Context};
use wm_common::WmEvent;

use super::sort_workspaces;
use crate::{
  models::Workspace, traits::CommonGetters, user_config::UserConfig,
  wm_state::WmState,
};

pub fn rename_workspace(
  workspace: &Workspace,
  name: &String,
  display_name: Option<String>,
  state: &WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let mut workspace_config = workspace.config();
  if name == &workspace_config.name {
    // When `name` is the same as target workspace, change only the display
    // name
    workspace_config.display_name = display_name;
  } else if let Some(other_workspace) = state.workspace_by_name(name) {
    // Do nothing if the name has been taken
    let config = other_workspace.config();
    bail!("The workspace \"{}\" already exists", config.name);
  } else {
    // Rename!
    name.clone_into(&mut workspace_config.name);
    workspace_config.display_name = display_name;
  }
  // Commit the change
  workspace.set_config(workspace_config);
  let monitor = workspace.monitor().context("No displayed workspace.")?;
  sort_workspaces(&monitor, config)?;
  state.emit_event(WmEvent::WorkspaceUpdated {
    updated_workspace: workspace.to_dto()?,
  });
  Ok(())
}
