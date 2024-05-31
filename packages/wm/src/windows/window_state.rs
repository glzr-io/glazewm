use serde::Serialize;

use crate::user_config::{FloatingStateConfig, FullscreenStateConfig};

/// Represents the various states a window can be in.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum WindowState {
  Floating(FloatingStateConfig),
  Fullscreen(FullscreenStateConfig),
  Minimized,
  Tiling,
}
