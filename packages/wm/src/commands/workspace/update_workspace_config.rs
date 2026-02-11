use anyhow::{bail, Context};
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
  let WorkspaceConfig {
    mut name,
    display_name,
    bind_to_monitor,
    keep_alive,
  } = workspace.config();

  let mut need_sort = false;

  // validate the workspace name change
  if let Some(new_name) = &new_config.name {
    if new_name != &name {
      if let Some(_other_workspace) = state.workspace_by_name(new_name) {
        bail!("The workspace \"{}\" already exists", new_name);
      }
      name.clone_from(new_name);
      need_sort = true;
    }
  }

  // the updated config
  let updated_config = WorkspaceConfig {
    name,
    display_name: if new_config.no_display_name {
      None
    } else {
      new_config.display_name.clone().or(display_name)
    },
    bind_to_monitor: new_config.bind_to_monitor.or(bind_to_monitor),
    keep_alive: new_config.keep_alive.unwrap_or(keep_alive),
  };

  // Commit the change
  workspace.set_config(updated_config);

  if need_sort {
    let monitor =
      workspace.monitor().context("No displayed workspace.")?;
    sort_workspaces(&monitor, config)?;
  }

  state.emit_event(WmEvent::WorkspaceUpdated {
    updated_workspace: workspace.to_dto()?,
  });

  Ok(())
}
