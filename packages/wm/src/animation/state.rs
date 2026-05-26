use std::{
  cell::Cell,
  time::{Duration, Instant},
};

use wm_common::EasingFunction;
use wm_platform::{OpacityValue, Rect};

use crate::animation::engine::{animation_progress_at, apply_easing};

/// State of an individual window animation.
#[derive(Clone, Debug)]
pub struct WindowAnimationState {
  /// Time of the first rendered frame.
  ///
  /// Lazily initialized on the first `eased_progress_at` call so the clock
  /// starts when the first frame is actually rendered (aligned to VSync)
  /// rather than when the animation struct is created mid-`platform_sync`.
  /// Without lazy init, a cold-start gap of 1–2 DWM frames causes the first
  /// rendered frame to already show non-zero progress, producing a visible
  /// jump at the start of the animation.
  start_time: Cell<Option<Instant>>,
  /// Time to wait before advancing progress.
  ///
  /// Used for staggered workspace-switch animations where each window starts
  /// at a different offset within the shared duration window.
  pub start_delay: Duration,
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
      start_delay: Duration::ZERO,
      duration: Duration::from_millis(u64::from(duration_ms)),
      easing,
      start_rect,
      target_rect,
      start_opacity: None,
      target_opacity: None,
    }
  }

  /// Sets the delay before this animation starts and returns `self`.
  #[allow(dead_code)]
  pub fn with_delay(mut self, delay: Duration) -> Self {
    self.start_delay = delay;
    self
  }

  /// Gets the eased progress in [0.0, 1.0] at an explicit `now` instant.
  ///
  /// Allows callers to supply a predictive timestamp (e.g. vsync wake-up time
  /// plus an estimated pipeline offset) so the computed position aligns with
  /// the DWM composition event rather than the moment this code runs.
  ///
  /// `start_delay` is applied before the duration window begins: if
  /// `elapsed < start_delay`, returns 0.0 without advancing the animation.
  /// All windows initialized on the same tick share the same `start_time`,
  /// so staggering is purely a function of each window's `start_delay`.
  ///
  /// Non-overshooting curves snap to 1.0 at 99% eased progress to avoid
  /// the "stuck at destination" look. Overshooting curves run to full
  /// wall-clock duration to preserve their bounce.
  pub fn eased_progress_at(&self, now: Instant) -> f32 {
    let start = self.start_time.get().unwrap_or_else(|| {
      self.start_time.set(Some(now));
      now
    });

    let elapsed = now.saturating_duration_since(start);
    if elapsed < self.start_delay {
      return 0.0;
    }

    // Shift the clock origin past the delay so the duration window begins
    // at `start + start_delay`. `animation_progress_at` uses
    // `saturating_duration_since`, so passing a future `effective_start` is
    // safe even if `now` precedes it on the first delayed tick.
    let effective_start = start + self.start_delay;
    let raw = animation_progress_at(effective_start, self.duration, now);
    let eased = apply_easing(raw, &self.easing);
    let done = if self.easing.can_overshoot() {
      raw == 1.0
    } else {
      raw == 1.0 || eased >= 0.99
    };
    if done { 1.0 } else { eased }
  }

  /// Gets the eased progress in [0.0, 1.0], snapping to 1.0 when complete.
  pub fn eased_progress(&self) -> f32 {
    self.eased_progress_at(Instant::now())
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
  /// when both values are needed in the same frame — `eased_progress` (which
  /// runs a Newton-Raphson solve) is computed only once.
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
