use anyhow::Context;
use wm_common::WmEvent;

use crate::{user_config::UserConfig, wm_state::WmState};

pub fn enable_binding_mode(
  name: &str,
  state: &mut WmState,
  config: &UserConfig,
) -> anyhow::Result<()> {
  let binding_mode = config
    .value
    .binding_modes
    .iter()
    .find(|config| name == config.name)
    .with_context(|| {
      format!("No binding mode found with the name '{}'.", name)
    })?;

  state.binding_modes = vec![binding_mode.clone()];

  state.emit_event(WmEvent::BindingModesChanged {
    new_binding_modes: state.binding_modes.clone(),
  });

  Ok(())
}
