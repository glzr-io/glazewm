use serde::Serialize;

use crate::user_config::{FloatingStateConfig, FullscreenStateConfig};

/// Represents the various states a window can be in.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowState {
  Floating(FloatingStateConfig),
  Fullscreen(FullscreenStateConfig),
  Minimized,
  Tiling,
}

/// Intermediate struct for serializing `WindowState`.
#[derive(Debug, Serialize)]
struct WindowStateDto {
  name: String,
  options: Option<WindowStateOptions>,
}

/// Intermediate enum for serializing `WindowState`.
#[derive(Debug, Serialize)]
#[serde(untagged)]
enum WindowStateOptions {
  Floating(FloatingStateConfig),
  Fullscreen(FullscreenStateConfig),
}

impl Serialize for WindowState {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let type_str = match self {
      WindowState::Floating(_) => "floating",
      WindowState::Fullscreen(_) => "fullscreen",
      WindowState::Minimized => "minimized",
      WindowState::Tiling => "tiling",
    };

    let options = match self {
      WindowState::Floating(floating) => {
        Some(WindowStateOptions::Floating(floating.clone()))
      }
      WindowState::Fullscreen(fullscreen) => {
        Some(WindowStateOptions::Fullscreen(fullscreen.clone()))
      }
      _ => None,
    };

    let dto = WindowStateDto {
      name: type_str.to_string(),
      options,
    };

    dto.serialize(serializer)
  }
}
