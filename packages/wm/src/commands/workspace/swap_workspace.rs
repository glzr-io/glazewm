use anyhow::Context;
use tracing::info;

use super::{activate_workspace};
use crate::{
  commands::{
    container::set_focused_descendant, workspace::deactivate_workspace,
  },
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn swap_workspace(
  monitor_a_index: usize,
  monitor_b_index: usize,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let monitors = state.monitors();
  let monitor_a = monitors
    .get(monitor_a_index)
    .with_context(|| format!("Monitor at {monitor_a_index} does not exist."))?;

  let monitor_b = monitors
    .get(monitor_b_index)
    .with_context(|| format!("Monitor at {monitor_b_index} does not exist."))?;

  let workspace_at_a = monitor_a
    .displayed_workspace()
    .context("No displayed workspace.")?;

  let workspace_at_b = monitor_b
    .displayed_workspace()
    .context("No displayed workspace.")?;



  Ok(())
}
