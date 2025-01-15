use serde::{Deserialize, Serialize};

use crate::{
  parsed_config::{
    FloatingStateConfig, FullscreenStateConfig, InitialWindowState,
  },
  ParsedConfig,
};

/// Represents the possible states a window can have.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WindowState {
  Floating(FloatingStateConfig),
  Fullscreen(FullscreenStateConfig),
  Minimized,
  Tiling,
}

impl WindowState {
  #[must_use]
  pub fn default_from_config(config: &ParsedConfig) -> Self {
    match config.window_behavior.initial_state {
      InitialWindowState::Tiling => Self::Tiling,
      InitialWindowState::Floating => Self::Floating(
        config.window_behavior.state_defaults.floating.clone(),
      ),
    }
  }

  #[must_use]
  pub fn is_same_state(&self, other: &Self) -> bool {
    std::mem::discriminant(self) == std::mem::discriminant(other)
  }
}
