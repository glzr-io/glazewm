use anyhow::Context;

use crate::{
  commands::monitor::{
    move_bounded_workspaces_to_new_monitor, move_workspace_to_monitor,
  },
  models::Workspace,
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Matches workspace placement to `general.multi_monitor_workspaces` for
/// the current monitor topology.
///
/// Idempotent: safe to call after any change to the monitor topology or to
/// the `multi_monitor_workspaces` option itself.
pub fn sync_workspaces_to_monitor_topology(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  if config.value.general.multi_monitor_workspaces {
    // Same second pass as `WmState::populate`: bind workspaces to
    // monitors and attach at least one workspace per monitor.
    for monitor in state.monitors() {
      move_bounded_workspaces_to_new_monitor(&monitor, state, config)?;
    }
  } else {
    consolidate_non_primary_workspaces_onto_primary(state, config)?;
  }

  Ok(())
}

/// Moves every workspace off non-primary monitors so only the logical
/// primary holds workspaces (`multi_monitor_workspaces: false`).
///
/// No-op when all workspaces already reside on the primary monitor.
pub fn consolidate_non_primary_workspaces_onto_primary(
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let primary_monitor = state
    .primary_monitor(config)
    .context("No primary monitor while consolidating workspaces.")?;

  let workspaces_on_secondary: Vec<Workspace> = state
    .monitors()
    .into_iter()
    .filter(|monitor| monitor.id() != primary_monitor.id())
    .flat_map(|monitor| monitor.workspaces())
    .collect();

  for workspace in workspaces_on_secondary {
    move_workspace_to_monitor(&workspace, &primary_monitor, state, config)?;
  }

  Ok(())
}
