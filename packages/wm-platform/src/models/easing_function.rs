use serde::{Deserialize, Serialize};

/// Supported easing functions for animations.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EasingFunction {
  Linear,
  #[default]
  EaseInOut,
  EaseIn,
  EaseOut,
  EaseInOutCubic,
  EaseInCubic,
  EaseOutCubic,
}

impl EasingFunction {
  /// Applies the easing function to a progress value (0.0 to 1.0).
  #[must_use]
  pub fn apply(&self, progress: f32) -> f32 {
    match self {
      Self::Linear => progress,
      Self::EaseInOut => {
        if progress < 0.5 {
          2.0 * progress * progress
        } else {
          -1.0 + (4.0 - 2.0 * progress) * progress
        }
      }
      Self::EaseIn => progress * progress,
      Self::EaseOut => progress * (2.0 - progress),
      EasingFunction::EaseInOutCubic => {
        if progress < 0.5 {
          4.0 * progress * progress * progress
        } else {
          1.0 - (-2.0 * progress + 2.0).powi(3) / 2.0
        }
      }
      Self::EaseInCubic => progress * progress * progress,
      Self::EaseOutCubic => 1.0 - (1.0 - progress).powi(3),
    }
  }
}
