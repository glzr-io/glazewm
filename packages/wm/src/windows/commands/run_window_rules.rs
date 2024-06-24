use anyhow::Context;

use crate::{
  containers::WindowContainer, user_config::UserConfig, wm_state::WmState,
};

pub fn run_window_rules(
  window: WindowContainer,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  Ok(())
}
