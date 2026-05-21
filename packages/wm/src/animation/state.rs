use std::{
  cell::Cell,
  time::{Duration, Instant},
};

use wm_common::EasingFunction;
use wm_platform::{OpacityValue, Rect};

use crate::animation::engine::{animation_progress, apply_easing};

/// State of an individual window animation.
#[derive(Clone, Debug)]
pub struct WindowAnimationState {
  /// Time of the first rendered frame.
  ///
  /// Lazily initialized on the first `eased_progress` call so the clock
  /// starts when the first frame is actually rendered (aligned to VSync)
  /// rather than when the animation struct is created mid-`platform_sync`.
  /// Without lazy init, a cold-start gap of 1–2 DWM frames causes the first
  /// rendered frame to already show non-zero progress, producing a visible
  /// jump at the start of the animation.
  start_time: Cell<Option<Instant>>,
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
    duration_ms: u32,
    easing: EasingFunction,
  ) -> Self {
    Self {
      start_time: Cell::new(None),
      duration: Duration::from_millis(u64::from(duration_ms)),
      easing,
      start_rect,
      target_rect,
      start_opacity: None,
      target_opacity: None,
    }
  }

  /// Gets the eased progress in [0.0, 1.0], snapping to 1.0 when complete.
  ///
  /// Non-overshooting curves snap to 1.0 at 99% eased progress to avoid
  /// the "stuck at destination" look. Overshooting curves run to full
  /// wall-clock duration to preserve their bounce.
  ///
  /// Lazily initializes `start_time` on the first call so elapsed time is
  /// measured from the first rendered frame rather than from animation
  /// creation time.
  pub fn eased_progress(&self) -> f32 {
    let start = self.start_time.get().unwrap_or_else(|| {
      let now = Instant::now();
      self.start_time.set(Some(now));
      now
    });
    let raw = animation_progress(start, self.duration);
    let eased = apply_easing(raw, &self.easing);
    let done = if self.easing.can_overshoot() {
      raw == 1.0
    } else {
      raw == 1.0 || eased >= 0.99
    };
    if done { 1.0 } else { eased }
  }

  /// Whether the animation has completed.
  pub fn is_complete(&self) -> bool {
    self.eased_progress() == 1.0
  }

  /// Gets the interpolated rect at the current animation progress.
  pub fn current_rect(&self) -> Rect {
    self.start_rect.interpolate(&self.target_rect, self.eased_progress())
  }

  /// Gets the interpolated rect and opacity in a single call.
  ///
  /// Prefer this over separate `current_rect` + `current_opacity` calls
  /// when both values are needed in the same frame — `effective_progress`
  /// (which runs a Newton-Raphson solve) is computed only once.
  pub fn current_state(&self) -> (Rect, Option<OpacityValue>) {
    let progress = self.eased_progress();
    let rect = self.start_rect.interpolate(&self.target_rect, progress);
    let opacity = match (&self.start_opacity, &self.target_opacity) {
      (Some(start), Some(end)) => Some(start.interpolate(end, progress)),
      _ => None,
    };
    (rect, opacity)
  }
}

