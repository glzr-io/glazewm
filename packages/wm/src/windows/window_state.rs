use serde::{Deserialize, Serialize};

use crate::user_config::{FloatingStateConfig, FullscreenStateConfig};

/// Represents the different possible states a window can have.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "name", rename_all = "snake_case")]
pub enum WindowState {
  Floating(FloatingStateConfig),
  Fullscreen(FullscreenStateConfig),
  Minimized,
  Tiling,
}
