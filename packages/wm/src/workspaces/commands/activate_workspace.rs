use anyhow::Context;

use crate::{
  common::TilingDirection,
  containers::{commands::attach_container, traits::PositionGetters},
  monitors::Monitor,
  user_config::{UserConfig, WorkspaceConfig},
  wm_event::WmEvent,
  wm_state::WmState,
  workspaces::Workspace,
};

/// Activates a workspace on the target monitor.
///
/// If no workspace name is provided, the first suitable workspace defined
/// in the user's config will be used.
pub fn activate_workspace(
  workspace_name: Option<String>,
  target_monitor: &Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspace_config =
    get_workspace_config(workspace_name, target_monitor, state, config)?;

  let tiling_direction =
    match target_monitor.height()? > target_monitor.width()? {
      true => TilingDirection::Vertical,
      false => TilingDirection::Horizontal,
    };

  let workspace = Workspace::new(
    workspace_config,
    config.value.gaps.outer_gap.clone(),
    tiling_direction,
  );

  // Attach the created workspace to the specified monitor.
  attach_container(
    workspace.clone().into(),
    &target_monitor.clone().into(),
    None,
  )?;

  state.emit_event(WmEvent::WorkspaceActivated {
    activated_workspace: workspace,
  });

  Ok(())
}

fn get_workspace_config(
  workspace_name: Option<String>,
  target_monitor: &Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WorkspaceConfig> {
  match workspace_name {
    Some(workspace_name) => {
      let found_config = state
        .workspaces()
        .iter()
        .find(|w| w.config().name == workspace_name)
        .with_context(|| {
          format!("Workspace with name {} doesn't exist.", workspace_name)
        })?
        .config();

      Ok(found_config)
    }
    None => {
      let inactive_config = config
        .workspace_config_for_monitor(&target_monitor, &state.workspaces())
        .context("No workspace config found for monitor.")?;

      Ok(inactive_config)
    }
  }
}
