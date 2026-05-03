use std::time::{Duration, Instant};

use wm_common::{AnimationTypeConfig, EasingFunction};
use wm_platform::{OpacityValue, Rect};

use crate::animation::engine::{animation_progress, apply_easing};

/// State of an individual window animation.
#[derive(Clone, Debug)]
pub struct WindowAnimationState {
  pub start_time: Instant,
  pub duration: Duration,
  pub easing: EasingFunction,

  // Position animation.
  pub start_rect: Rect,
  pub target_rect: Rect,

  // Opacity animation; `None` when fade is disabled.
  pub start_opacity: Option<OpacityValue>,
  pub target_opacity: Option<OpacityValue>,
}

impl WindowAnimationState {
  /// Creates a new movement animation.
  pub fn new_movement(
    start_rect: Rect,
    target_rect: Rect,
    config: &AnimationTypeConfig,
  ) -> Self {
    Self {
      start_time: Instant::now(),
      duration: Duration::from_millis(u64::from(config.duration_ms)),
      easing: config.easing.clone(),
      start_rect,
      target_rect,
      start_opacity: None,
      target_opacity: None,
    }
  }

  /// Gets the effective eased progress, returning 1.0 when the animation
  /// is considered complete.
  ///
  /// For non-overshooting easing functions, snaps to 1.0 once the eased
  /// value reaches 0.99. Decelerating curves (e.g. `EaseOutCubic`) spend
  /// roughly 22% of wall time covering the last 1% of distance, which
  /// makes windows look "stuck" at their destination. `EaseOutSpring` and
  /// `CubicBezier` can overshoot past 1.0, so they always run to the full
  /// wall-clock duration to preserve the overshoot/bounce.
  fn effective_progress(&self) -> f32 {
    let raw = animation_progress(self.start_time, self.duration);
    let eased = apply_easing(raw, &self.easing);
    let done = match &self.easing {
      EasingFunction::EaseOutSpring | EasingFunction::CubicBezier(..) => {
        raw == 1.0
      }
      _ => raw == 1.0 || eased >= 0.99,
    };
    if done { 1.0 } else { eased }
  }

  /// Whether the animation has completed.
  pub fn is_complete(&self) -> bool {
    self.effective_progress() == 1.0
  }

  /// Gets the interpolated rect at the current animation progress.
  pub fn current_rect(&self) -> Rect {
    self.start_rect.interpolate(&self.target_rect, self.effective_progress())
  }

  /// Gets the interpolated opacity at the current animation progress.
  pub fn current_opacity(&self) -> Option<OpacityValue> {
    let (Some(start), Some(end)) =
      (&self.start_opacity, &self.target_opacity)
    else {
      return None;
    };
    Some(start.interpolate(end, self.effective_progress()))
  }
}

