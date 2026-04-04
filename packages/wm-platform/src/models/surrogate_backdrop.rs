use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

use crate::Color;

/// Backdrop style for the surrogate overlay window used during resize
/// animations.
///
/// Controls what is rendered in the area of the surrogate that extends
/// beyond the DWM thumbnail (i.e. the region growing toward the target
/// rect).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SurrogateBackdrop {
  /// Windows Acrylic blur-behind (default). Requires Windows 10 1803+;
  /// degrades gracefully on older versions.
  Acrylic,
  /// Sample the average color along the window edges at animation start
  /// and fill the backdrop with that color, making the extension appear
  /// as a natural continuation of the window content. Falls back to
  /// `Acrylic` when pixel capture fails.
  Auto,
  /// Flat fill using the specified color (e.g. `"#1a1a1a"` or
  /// `"#1a1a1aCC"` with alpha).
  Color(Color),
}

impl Default for SurrogateBackdrop {
  fn default() -> Self {
    Self::Acrylic
  }
}

impl FromStr for SurrogateBackdrop {
  type Err = crate::ParseError;

  fn from_str(s: &str) -> Result<Self, crate::ParseError> {
    match s {
      "acrylic" => Ok(Self::Acrylic),
      "auto" => Ok(Self::Auto),
      _ => Color::from_str(s).map(Self::Color),
    }
  }
}

impl<'de> Deserialize<'de> for SurrogateBackdrop {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Self::from_str(&s).map_err(serde::de::Error::custom)
  }
}
