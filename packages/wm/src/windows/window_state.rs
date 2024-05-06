use crate::user_config::{FloatingStateConfig, FullscreenStateConfig};

/// Represents the various states a window can be in.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowState {
  Floating(FloatingStateConfig),
  Fullscreen(FullscreenStateConfig),
  Minimized,
  Tiling,
}
