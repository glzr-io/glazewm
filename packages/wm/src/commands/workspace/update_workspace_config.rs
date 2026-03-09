use anyhow::Context;
use wm_common::{InvokeUpdateWorkspaceConfig, WmEvent, WorkspaceConfig};

use super::sort_workspaces;
use crate::{
  models::Workspace, traits::CommonGetters, user_config::UserConfig,
  wm_state::WmState,
};

pub fn update_workspace_config(
  workspace: &Workspace,
  state: &WmState,
  config: &UserConfig,
  new_config: &InvokeUpdateWorkspaceConfig,
) -> anyhow::Result<()> {
  let current_config = workspace.config();

  // Validate the workspace name change.
  if let Some(new_name) = &new_config.name {
    if new_name != &current_config.name {
      if let Some(_other_workspace) = state.workspace_by_name(new_name) {
        anyhow::bail!("The workspace \"{}\" already exists", new_name);
      }
    }
  }

  // Update the config with the incoming values.
  let updated_config = WorkspaceConfig {
    name: new_config
      .name
      .clone()
      .unwrap_or(current_config.name.clone()),
    display_name: new_config
      .display_name
      .clone()
      .or(current_config.display_name.clone()),
    bind_to_monitor: new_config
      .bind_to_monitor
      .or(current_config.bind_to_monitor),
    keep_alive: new_config.keep_alive.unwrap_or(current_config.keep_alive),
  };

  workspace.set_config(updated_config);

  sort_workspaces(
    &workspace.monitor().context("No displayed workspace.")?,
    config,
  )?;

  // TODO: Re-assign bound workspaces to their respective monitors.

  state.emit_event(WmEvent::WorkspaceUpdated {
    updated_workspace: workspace.to_dto()?,
  });

  Ok(())
}
