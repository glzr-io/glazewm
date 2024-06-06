use anyhow::Context;
use tracing::{info, warn};

use crate::{
  containers::traits::{CommonGetters, TilingSizeGetters},
  user_config::UserConfig,
  wm_event::WmEvent,
  wm_state::WmState,
};

pub fn reload_config(
  state: &mut WmState,
  config: &mut UserConfig,
) -> anyhow::Result<()> {
  info!("Config reloaded.");

  // Re-evaluate user config file and set its values in state.
  let handle = tokio::runtime::Handle::current();
  handle.block_on(config.reload())?;

  // TODO: Run window rules on all windows.

  // Update configs of active workspaces.
  let workspaces = state.workspaces();
  for workspace in &workspaces {
    let workspace_config = config
      .value
      .workspaces
      .iter()
      .find(|config| config.name == workspace.config().name);

    match workspace_config {
      Some(workspace_config) => {
        workspace.set_config(workspace_config.clone());
      }
      // When the workspace config is not found, the current name of the
      // workspace has been removed. So, we reassign the first suitable
      // workspace config to the workspace.
      None => {
        let monitor = workspace.monitor().context("No monitor.")?;
        let inactive_config =
          config.workspace_config_for_monitor(&monitor, &workspaces);

        if let Some(inactive_config) = inactive_config {
          workspace.set_config(inactive_config.clone());
        } else {
          warn!(
            "Unable to update workspace config. No available workspace configs."
          );
        }
      }
    }
  }

  // Update inner gaps of tiling containers.
  for container in state
    .root_container
    .self_and_descendants()
    .into_iter()
    .filter_map(|c| c.as_tiling_container().ok())
  {
    container.set_inner_gap(config.value.gaps.inner_gap.clone());
  }

  // Clear active binding modes.
  state.binding_modes = Vec::new();

  // Redraw full container tree.
  let root_container = state.root_container.clone();
  state.add_container_to_redraw(root_container.into());

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

  Ok(())
}
