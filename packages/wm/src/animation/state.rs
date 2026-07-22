use std::{
  cell::Cell,
  time::{Duration, Instant},
};

use wm_common::EasingFunction;
use wm_platform::{OpacityValue, Rect};

use crate::animation::engine::{animation_progress_at, apply_easing};

/// Residual rect travel distance, in pixels, at which a non-overshooting
/// animation completes early.
///
/// Kept below one pixel so the single-frame snap from the early-completion
/// position to the exact target is imperceptible for any travel distance.
/// Mirrors `WS_COMPLETE_THRESHOLD_PX` used by the workspace-switch driver.
const COMPLETE_THRESHOLD_PX: f32 = 1.0;

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
    } else if raw == 1.0 {
      true
    } else {
      // Decelerating easing spends a large fraction of its wall-clock
      // duration covering the final sliver of distance, which looks "stuck"
      // at the destination — so complete early. Gating that completion on a
      // fixed residual *pixel* distance (rather than a fixed eased fraction)
      // keeps the completion-frame snap sub-pixel regardless of travel
      // distance: a fixed `eased >= 0.99` would snap ~1% of the travel, which
      // is imperceptible for a short move but a visible 10-20px jump at the
      // end of an open/close slide spanning a whole window dimension. Mirrors
      // `WS_COMPLETE_THRESHOLD_PX` in the workspace-switch driver.
      let max_travel = self.max_travel_px();
      if max_travel > 0.0 {
        (1.0 - eased) * max_travel <= COMPLETE_THRESHOLD_PX
      } else {
        // No positional travel (e.g. an opacity-only fade): there is no
        // pixel distance to gate on, so fall back to the eased fraction.
        eased >= 0.99
      }
    };
    if done { 1.0 } else { eased }
  }

  /// Gets the eased progress in [0.0, 1.0], snapping to 1.0 when complete.
  pub fn eased_progress(&self) -> f32 {
    self.eased_progress_at(Instant::now())
  }

  /// Remaining wall-clock time until the animation's duration window
  /// elapses at `now`.
  ///
  /// Returns the full `start_delay + duration` when the animation has not
  /// rendered its first frame yet (`start_time` unset). Used to schedule the
  /// mid-animation handoff of the real window to its final rect.
  pub fn remaining_at(&self, now: Instant) -> Duration {
    match self.start_time.get() {
      Some(start) => (start + self.start_delay + self.duration)
        .saturating_duration_since(now),
      None => self.start_delay + self.duration,
    }
  }

  /// Largest per-axis rect travel distance, in pixels, between the start and
  /// target rects.
  ///
  /// Returns the maximum of the absolute position and size deltas, giving an
  /// upper bound on how far any edge of the window moves over the animation.
  /// Returns `0` for opacity-only animations whose start and target rects are
  /// identical.
  fn max_travel_px(&self) -> f32 {
    let dx = (self.target_rect.x() - self.start_rect.x()).abs();
    let dy = (self.target_rect.y() - self.start_rect.y()).abs();
    let dw = (self.target_rect.width() - self.start_rect.width()).abs();
    let dh = (self.target_rect.height() - self.start_rect.height()).abs();
    #[allow(clippy::cast_precision_loss)]
    {
      dx.max(dy).max(dw).max(dh) as f32
    }
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
    self.current_state_at(Instant::now())
  }

  /// Gets the interpolated rect and opacity at an explicit `now` instant.
  ///
  /// Like [`current_state`], but evaluates progress at a caller-supplied
  /// predictive timestamp (e.g. a vsync wake-up led forward by a fraction of
  /// a frame) so the computed position aligns with the next DWM composition
  /// rather than the moment this code runs.
  ///
  /// [`current_state`]: WindowAnimationState::current_state
  pub fn current_state_at(
    &self,
    now: Instant,
  ) -> (Rect, Option<OpacityValue>) {
    let progress = self.eased_progress_at(now);
    let rect = self.start_rect.interpolate(&self.target_rect, progress);
    let opacity = match (&self.start_opacity, &self.target_opacity) {
      (Some(start), Some(end)) => Some(start.interpolate(end, progress)),
      _ => None,
    };
    (rect, opacity)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use wm_platform::Rect;

  /// A cubic bezier with collinear, evenly-spaced control points is exactly
  /// the identity curve, so `eased == raw`. Used to make completion behaviour
  /// deterministic in tests.
  fn linear() -> EasingFunction {
    EasingFunction::CubicBezier(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0)
  }

  /// A long slide must not snap to the target while it is still many pixels
  /// away: at 99% progress over a 10000px slide the residual is 100px, so the
  /// animation reports the eased value rather than completing.
  #[test]
  fn long_slide_does_not_snap_at_ninety_nine_percent() {
    let anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(10_000, 0, 100, 100),
      10_000,
      linear(),
    );

    let t0 = Instant::now();
    // First call anchors `start_time` at `t0`.
    assert_eq!(anim.eased_progress_at(t0), 0.0);

    let progress = anim.eased_progress_at(t0 + Duration::from_millis(9_900));
    assert!(progress < 1.0, "expected no early snap, got {progress}");
    assert!((progress - 0.99).abs() < 1e-3, "got {progress}");
  }

  /// A slide completes once it is within one pixel of the target: at 99.9%
  /// progress over a 100px slide the residual is 0.1px, so it snaps to 1.0.
  #[test]
  fn long_slide_completes_within_one_pixel() {
    let anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(100, 0, 100, 100),
      10_000,
      linear(),
    );

    let t0 = Instant::now();
    assert_eq!(anim.eased_progress_at(t0), 0.0);

    let progress = anim.eased_progress_at(t0 + Duration::from_millis(9_990));
    assert_eq!(progress, 1.0);
  }

  /// An opacity-only animation has zero positional travel, so completion falls
  /// back to the eased fraction: it is incomplete just below 99% and complete
  /// at/above it.
  #[test]
  fn opacity_only_completes_on_eased_fraction() {
    let anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(0, 0, 100, 100),
      100,
      linear(),
    );

    let t0 = Instant::now();
    assert_eq!(anim.eased_progress_at(t0), 0.0);

    let before = anim.eased_progress_at(t0 + Duration::from_millis(98));
    assert!((before - 0.98).abs() < 1e-3, "got {before}");

    let after = anim.eased_progress_at(t0 + Duration::from_millis(99));
    assert_eq!(after, 1.0);
  }

  /// `start_delay` holds the animation at progress 0.0 until the delay elapses
  /// (used by the window-open paint grace period).
  #[test]
  fn start_delay_holds_at_zero() {
    let anim = WindowAnimationState::new_movement(
      Rect::from_xy(0, 0, 100, 100),
      Rect::from_xy(1_000, 0, 100, 100),
      100,
      linear(),
    )
    .with_delay(Duration::from_millis(30));

    let t0 = Instant::now();
    // Anchors `start_time`; still within the delay window.
    assert_eq!(anim.eased_progress_at(t0), 0.0);
    assert_eq!(anim.eased_progress_at(t0 + Duration::from_millis(20)), 0.0);

    // 30ms in, the duration window has just begun (progress ~0, not snapped).
    let after_delay =
      anim.eased_progress_at(t0 + Duration::from_millis(30));
    assert!(after_delay < 0.01, "got {after_delay}");

    // 30ms delay + 50ms into the 100ms duration → ~50% progress.
    let mid = anim.eased_progress_at(t0 + Duration::from_millis(80));
    assert!((mid - 0.5).abs() < 1e-2, "got {mid}");
  }
}
