use anyhow::Context;
use tracing::{info, warn};

use crate::{
  app_command::InvokeCommand, common::platform::Platform, containers::traits::{CommonGetters, TilingSizeGetters}, user_config::{ParsedConfig, UserConfig, WindowRuleEvent}, windows::{commands::run_window_rules, traits::WindowGetters}, wm_event::WmEvent, wm_state::WmState, workspaces::commands::sort_workspaces
};

pub fn reload_config(
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  info!("Config reloaded.");

  // Keep reference to old config for comparison.
  let old_config = config.value.clone();

  // Re-evaluate user config file and set its values in state.
  config.reload()?;

  // Re-run window rules on all active windows.
  for window in state.windows() {
    window.set_done_window_rules(Vec::new());
    run_window_rules(window, WindowRuleEvent::Manage, state, config)?;
  }

  update_workspace_configs(state, config)?;

  update_container_gaps(state, config);

  update_window_effects(&old_config, state, config)?;

  // Clear active binding modes.
  state.binding_modes = Vec::new();

  // Redraw full container tree.
  let root_container = state.root_container.clone();
  state
    .pending_sync
    .containers_to_redraw
    .push(root_container.into());

  // Emit the updated config.
  state.emit_event(WmEvent::UserConfigChanged {
    config_path: config
      .path
      .to_str()
      .context("Invalid config path.")?
      .to_string(),
    config_string: config.value_str.clone(),
    parsed_config: config.value.clone(),
  });

  // Run config reload commands.
  InvokeCommand::run_multiple(
    config.value.general.config_reload_commands.clone(),
    state.focused_container().context("No focused container.")?,
    state,
    config,
  )?;

  Ok(())
}

/// Update configs of active workspaces.
fn update_workspace_configs(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let workspaces = state.workspaces();

  for workspace in &workspaces {
    let monitor = workspace.monitor().context("No monitor.")?;

    let workspace_config = config
      .value
      .workspaces
      .iter()
      .find(|config| config.name == workspace.config().name)
      .or_else(|| {
        // When the workspace config is not found, the current name of the
        // workspace has been removed. So, we reassign the first suitable
        // workspace config to the workspace.
        config
          .workspace_config_for_monitor(&monitor, &workspaces)
          .or_else(|| config.next_inactive_workspace_config(&workspaces))
      });

    match workspace_config {
      None => {
        warn!(
          "Unable to update workspace config. No available workspace configs."
        );
      }
      Some(workspace_config) => {
        if *workspace_config != workspace.config() {
          workspace.set_config(workspace_config.clone());

          sort_workspaces(monitor, config)?;

          state.emit_event(WmEvent::WorkspaceUpdated {
            updated_workspace: workspace.to_dto()?,
          });
        }
      }
    }
  }

  Ok(())
}

/// Updates outer gap of workspaces and inner gaps of tiling containers.
fn update_container_gaps(state: &mut WmState, config: &UserConfig) {
  let tiling_containers = state
    .root_container
    .self_and_descendants()
    .into_iter()
    .filter_map(|container| container.as_tiling_container().ok());

  for container in tiling_containers {
    container.set_gaps_config(config.value.gaps.clone());
  }

  for workspace in state.workspaces() {
    workspace.set_gaps_config(config.value.gaps.clone());
  }
}

fn update_window_effects(
  old_config: &ParsedConfig,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let focused_container =
    state.focused_container().context("No focused container.")?;


  let general_config = &config.value.general;
  let old_general_config = &old_config.general;

  let window_effects = &config.value.window_effects;
  let old_window_effects = &old_config.window_effects;

  if general_config.window_animations != old_general_config.window_animations {
    Platform::set_window_animations_enabled(general_config.window_animations)?;
  }

  // Window border effects are left at system defaults if disabled in the
  // config. However, when transitioning from colored borders to having
  // them disabled, it's best to reset to the system defaults.
  if !window_effects.focused_window.border.enabled
    && old_window_effects.focused_window.border.enabled
  {
    if let Ok(window) = focused_container.as_window_container() {
      _ = window.native().set_border_color(None);
    }
  }

  if !window_effects.other_windows.border.enabled
    && old_window_effects.other_windows.border.enabled
  {
    let unfocused_windows = state
      .windows()
      .into_iter()
      .filter(|window| window.id() != focused_container.id());

    for window in unfocused_windows {
      _ = window.native().set_border_color(None);
    }
  }

  state.pending_sync.reset_window_effects = true;

  Ok(())
}
