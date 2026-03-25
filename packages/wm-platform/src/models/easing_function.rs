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

/// Interpolates a value with an easing function applied.
pub fn interpolate_with_easing<T>(
  start: &T,
  end: &T,
  progress: f32,
  easing: &EasingFunction,
  interpolate_fn: impl Fn(&T, &T, f32) -> T,
) -> T {
  let eased_progress = easing.apply(progress);
  interpolate_fn(start, end, eased_progress)
}

#[cfg(test)]
mod tests {
  use crate::Rect;

  #[test]
  fn test_interpolate_rect() {
    let start = Rect::from_xy(0, 0, 100, 100);
    let end = Rect::from_xy(100, 100, 200, 200);

    let mid = start.interpolate(&end, 0.5);
    assert_eq!(mid.x(), 50);
    assert_eq!(mid.y(), 50);
    assert_eq!(mid.width(), 150);
    assert_eq!(mid.height(), 150);
  }

  #[test]
  fn test_scale_rect_from_center() {
    let rect = Rect::from_xy(100, 100, 200, 200);
    let scaled = rect.scale_from_center(0.5);

    assert_eq!(scaled.width(), 100);
    assert_eq!(scaled.height(), 100);
    assert_eq!(scaled.x(), 150);
    assert_eq!(scaled.y(), 150);
  }
}
