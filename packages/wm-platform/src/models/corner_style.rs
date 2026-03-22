use serde::{Deserialize, Serialize};

/// Corner style of a window's frame.
///
/// # Platform-specific
///
/// Only has an effect on Windows 11.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CornerStyle {
  #[default]
  Default,
  Square,
  Rounded,
  SmallRounded,
}
