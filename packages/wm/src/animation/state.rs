use std::time::{Duration, Instant};
use wm_common::{
  AnimationEffectsConfig, AnimationTypeConfig, EasingFunction, OpacityValue,
  Rect,
};

use crate::animation::engine::{animation_progress, interpolate_with_easing};

/// Type of animation being performed.
#[derive(Clone, Debug)]
pub enum AnimationType {
  Movement,
  Open,
}

/// State of an individual window animation.
#[derive(Clone, Debug)]
pub struct WindowAnimationState {
  pub animation_type: AnimationType,
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
      animation_type: AnimationType::Movement,
      start_time: Instant::now(),
      duration: Duration::from_millis(u64::from(config.duration_ms)),
      easing: config.easing.clone(),
      start_rect,
      target_rect,
      start_opacity: None,
      target_opacity: None,
    }
  }

  /// Creates a new open animation.
  ///
  /// The overlay window stays at `target_rect` for the duration; only the DWM
  /// thumbnail opacity is animated (fade-in via `DWM_TNP_OPACITY`). Scale and
  /// slide effects require per-frame `SetWindowPos` on the real window and are
  /// therefore not applied — only the fade component of `animation_type` is
  /// used.
  pub fn new_open(
    target_rect: Rect,
    config: &AnimationEffectsConfig,
  ) -> Self {
    Self {
      animation_type: AnimationType::Open,
      start_time: Instant::now(),
      duration: Duration::from_millis(u64::from(config.duration_ms)),
      easing: config.easing.clone(),
      start_rect: target_rect.clone(),
      target_rect,
      start_opacity: config
        .animation_type
        .has_fade()
        .then(|| OpacityValue::from_alpha(0)),
      target_opacity: config
        .animation_type
        .has_fade()
        .then(|| OpacityValue::from_alpha(255)),
    }
  }

  /// Gets the current animation progress (0.0 to 1.0).
  pub fn progress(&self) -> f32 {
    animation_progress(self.start_time, self.duration)
  }

  /// Whether the animation has completed.
  /// NOTE: Progress is clamped to [0.0, 1.0], so exact comparison is safe.
  pub fn is_complete(&self) -> bool {
    self.progress() == 1.0
  }

  /// Gets the interpolated rect at the current animation progress.
  pub fn current_rect(&self) -> Rect {
    let progress = self.progress();
    interpolate_with_easing(
      &self.start_rect,
      &self.target_rect,
      progress,
      &self.easing,
      |start, end, eased_progress| start.interpolate(end, eased_progress),
    )
  }

  /// Gets the interpolated opacity at the current animation progress.
  pub fn current_opacity(&self) -> Option<OpacityValue> {
    if let (Some(start), Some(end)) = (&self.start_opacity, &self.target_opacity)
    {
      let progress = self.progress();
      Some(interpolate_with_easing(
        start,
        end,
        progress,
        &self.easing,
        |start, end, eased_progress| start.interpolate(end, eased_progress),
      ))
    } else {
      None
    }
  }
}

