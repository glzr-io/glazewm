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
  workspace_name: Option<&str>,
  target_monitor: &Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspace_config =
    workspace_config(workspace_name, target_monitor, state, config)?;

  let monitor_rect = target_monitor.to_rect()?;
  let tiling_direction = match monitor_rect.height() > monitor_rect.width()
  {
    true => TilingDirection::Vertical,
    false => TilingDirection::Horizontal,
  };

  let workspace = Workspace::new(
    workspace_config.clone(),
    config.value.gaps.outer_gap.clone(),
    tiling_direction,
  );

  // Attach the created workspace to the specified monitor.
  attach_container(
    &workspace.clone().into(),
    &target_monitor.clone().into(),
    None,
  )?;

  state.emit_event(WmEvent::WorkspaceActivated {
    activated_workspace: workspace.to_dto()?,
  });

  Ok(())
}

/// Gets config for the workspace to activate.
fn workspace_config(
  workspace_name: Option<&str>,
  target_monitor: &Monitor,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<WorkspaceConfig> {
  match workspace_name {
    Some(workspace_name) => {
      let found_config = config
        .inactive_workspace_configs(&state.workspaces())
        .into_iter()
        .find(|config| config.name == workspace_name)
        .with_context(|| {
          format!("Workspace with name {} doesn't exist.", workspace_name)
        })?;

      Ok(found_config.clone())
    }
    None => {
      let inactive_config = config
        .workspace_config_for_monitor(&target_monitor, &state.workspaces())
        .context("No workspace config found for monitor.")?;

      Ok(inactive_config.clone())
    }
  }
}
