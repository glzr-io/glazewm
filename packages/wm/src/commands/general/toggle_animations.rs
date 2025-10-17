use wm_common::WmEvent;

use crate::{user_config::UserConfig, wm_state::WmState};

/// Toggles animations on or off.
pub fn toggle_animations(config: &mut UserConfig, state: &mut WmState) {
  let is_enabled = !config.value.animations.enabled;
  config.value.animations.enabled = is_enabled;

  state.emit_event(WmEvent::AnimationsChanged {
    animations_enabled: is_enabled,
  });
}

