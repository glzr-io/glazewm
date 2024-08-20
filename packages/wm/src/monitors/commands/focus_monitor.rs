use crate::{
  containers::traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
  workspaces::{commands::focus_workspace, WorkspaceTarget},
};

/// Focuses a monitor by a given monitor index.
pub fn focus_monitor(
  target: &usize,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let monitors = state.monitors();

  let target_monitor = monitors.get(target.clone());

  // if there are fewer monitors than the index provided error out and bail
  // early from function
  if target_monitor.is_none() {
    anyhow::bail!("target index greater than number of monitors");
  }

  let target_monitor = target_monitor.unwrap();

  if target_monitor.has_focus(None) {
    return Ok(());
  }

  let displayed_workspace = target_monitor.displayed_workspace().unwrap();

  focus_workspace(
    WorkspaceTarget::Name(displayed_workspace.config().name),
    state,
    config,
  )?;

  Ok(())
}
