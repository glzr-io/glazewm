use std::time::{Duration, Instant};
use wm_common::EasingFunction;

/// Calculates the current progress of an animation (0.0 to 1.0).
#[cfg(test)]
pub fn animation_progress(
  start_time: Instant,
  duration: Duration,
) -> f32 {
  animation_progress_at(start_time, duration, Instant::now())
}

/// Calculates the animation progress at an explicit `now` instant (0.0 to
/// 1.0).
///
/// Allows callers to supply a predictive timestamp (e.g. vsync wake-up time
/// plus an estimated pipeline offset) so the computed position aligns with
/// the DWM composition event rather than the moment `update_internal` runs.
/// Uses `saturating_duration_since` so a `now` that precedes `start_time`
/// (possible on the first frame when `pipeline_offset` > elapsed) returns
/// `0.0` instead of panicking.
pub fn animation_progress_at(
  start_time: Instant,
  duration: Duration,
  now: Instant,
) -> f32 {
  let elapsed = now.saturating_duration_since(start_time);

  if elapsed >= duration {
    return 1.0;
  }

  #[allow(clippy::cast_precision_loss)]
  let progress = elapsed.as_millis() as f32 / duration.as_millis() as f32;
  progress.clamp(0.0, 1.0)
}

/// Applies an easing function to a linear progress value (0.0 to 1.0).
pub fn apply_easing(progress: f32, easing: &EasingFunction) -> f32 {
  match easing {
    EasingFunction::EaseOutSpring => ease_out_spring(progress),
    EasingFunction::CubicBezier(x1, y1, x2, y2) => {
      cubic_bezier(*x1, *y1, *x2, *y2, progress)
    }
  }
}

/// Evaluates a CSS cubic bezier at the given `x` progress (0.0 to 1.0).
///
/// Control points `(x1, y1)` and `(x2, y2)` define the curve between the
/// implicit anchors `(0, 0)` and `(1, 1)`. Uses Newton-Raphson iteration
/// to find the curve parameter `t` such that `Bx(t) = x`, then returns
/// `By(t)`.
fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
  let cx = 3.0 * x1;
  let bx = 3.0 * (x2 - x1) - cx;
  let ax = 1.0 - cx - bx;

  let cy = 3.0 * y1;
  let by_ = 3.0 * (y2 - y1) - cy;
  let ay = 1.0 - cy - by_;

  let sample_x = |t: f32| ((ax * t + bx) * t + cx) * t;
  let sample_dx = |t: f32| (3.0 * ax * t + 2.0 * bx) * t + cx;
  let sample_y = |t: f32| ((ay * t + by_) * t + cy) * t;

  let mut t = x;
  for _ in 0..8 {
    let dx = sample_dx(t);
    if dx.abs() < 1e-6 {
      break;
    }
    t = (t - (sample_x(t) - x) / dx).clamp(0.0, 1.0);
  }

  sample_y(t)
}

/// Exponentially-decaying spring easing function.
///
/// Produces an underdamped spring effect: the value overshoots past 1.0,
/// oscillates, and settles. Runs to full wall-clock duration (not cut off
/// at 99%) to preserve the bounce.
fn ease_out_spring(t: f32) -> f32 {
  if t <= 0.0 {
    return 0.0;
  }
  if t >= 1.0 {
    return 1.0;
  }
  let c4 = (2.0 * std::f32::consts::PI) / 2.0;
  2.0f32.powf(-12.0 * t) * ((t * 4.0 - 4.5) * c4).sin() * -1.0 + 1.0
}

#[cfg(test)]
mod tests {
  use super::*;
  use wm_platform::Rect;

  #[test]
  fn test_animation_progress() {
    let start = Instant::now() - Duration::from_millis(50);
    let duration = Duration::from_millis(100);

    let progress = animation_progress(start, duration);
    assert!((progress - 0.5).abs() < 0.1);
  }

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
}

