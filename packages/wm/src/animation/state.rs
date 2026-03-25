use std::time::{Duration, Instant};

use wm_common::{
  AnimationEffectsConfig, AnimationTypeConfig, EasingFunction,
};
use wm_platform::{interpolate_with_easing, OpacityValue, Rect};

/// State of an individual window animation.
#[derive(Clone, Debug)]
pub struct WindowAnimationState {
  pub start_time: Instant,
  pub duration: Duration,
  pub easing: EasingFunction,

  /// Start and target positions for the animated window.
  pub start_rect: Rect,
  pub target_rect: Rect,

  /// Start and target opacity for fade-in animations, or `None` if no
  /// opacity animation is active.
  pub start_opacity: Option<OpacityValue>,
  pub target_opacity: Option<OpacityValue>,
}

impl WindowAnimationState {
  /// Creates a new movement animation between two rects.
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

  /// Creates a new open animation, optionally with scale and/or fade
  /// effects as configured.
  pub fn new_open(
    target_rect: Rect,
    config: &AnimationEffectsConfig,
  ) -> Self {
    let start_rect = if config.animation_type.has_scale() {
      target_rect.scale_from_center(0.9)
    } else {
      target_rect.clone()
    };

    let (start_opacity, target_opacity) =
      if config.animation_type.has_fade() {
        (
          Some(OpacityValue::from_alpha(0)),
          Some(OpacityValue::from_alpha(255)),
        )
      } else {
        (None, None)
      };

    Self {
      start_time: Instant::now(),
      duration: Duration::from_millis(u64::from(config.duration_ms)),
      easing: config.easing.clone(),
      start_rect,
      target_rect,
      start_opacity,
      target_opacity,
    }
  }

  /// Returns the normalized animation progress in `[0.0, 1.0]`.
  pub fn progress(&self) -> f32 {
    let elapsed = self.start_time.elapsed();

    if elapsed >= self.duration {
      1.0
    } else {
      #[allow(clippy::cast_precision_loss)]
      let progress =
        elapsed.as_millis() as f32 / self.duration.as_millis() as f32;

      progress.clamp(0.0, 1.0)
    }
  }

  /// Whether the animation has completed.
  pub fn is_complete(&self) -> bool {
    // Progress is clamped to [0.0, 1.0], so exact comparison is safe.
    self.progress() == 1.0
  }

  /// Returns the interpolated rect at the current animation progress.
  pub fn current_rect(&self) -> Rect {
    let progress = self.progress();
    interpolate_with_easing(
      &self.start_rect,
      &self.target_rect,
      progress,
      &self.easing,
      |start, end, p| start.interpolate(end, p),
    )
  }

  /// Returns the interpolated opacity at the current animation progress,
  /// or `None` if no opacity animation is active.
  pub fn current_opacity(&self) -> Option<OpacityValue> {
    let (start, end) =
      (self.start_opacity.as_ref()?, self.target_opacity.as_ref()?);
    let progress = self.progress();
    Some(interpolate_with_easing(
      start,
      end,
      progress,
      &self.easing,
      |s, e, p| s.interpolate(e, p),
    ))
  }
}
