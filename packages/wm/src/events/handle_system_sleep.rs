use anyhow::Context;
use tracing::info;

use crate::{
  user_config::UserConfig,
  wm_state::WmState,
};

pub fn handle_system_sleep(
  state: &mut WmState,
  _config: &UserConfig,
) -> anyhow::Result<()> {
  info!("System entering sleep, saving monitor assignments");

  // Save current monitor assignments
  state.save_monitor_assignments()?;

  Ok(())
}
