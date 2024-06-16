use serde::{Deserialize, Serialize};

use crate::user_config::{
  FloatingStateConfig, FullscreenStateConfig, InitialWindowState,
  UserConfig,
};

/// Represents the different possible states a window can have.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum WindowState {
  Floating(FloatingStateConfig),
  Fullscreen(FullscreenStateConfig),
  Minimized,
  Tiling,
}

impl WindowState {
  pub fn default_from_config(config: &UserConfig) -> Self {
    match config.value.window_behavior.initial_state {
      InitialWindowState::Tiling => WindowState::Tiling,
      InitialWindowState::Floating => WindowState::Floating(
        config.value.window_behavior.state_defaults.floating.clone(),
      ),
    }
  }
}
