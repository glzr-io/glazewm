use std::time::{Duration, Instant};
use wm_common::EasingFunction;

/// Calculates the current progress of an animation (0.0 to 1.0).
pub fn animation_progress(
  start_time: Instant,
  duration: Duration,
) -> f32 {
  let elapsed = start_time.elapsed();

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
    EasingFunction::Linear => progress,
    EasingFunction::EaseInOut => ease_in_out(progress),
    EasingFunction::EaseIn => ease_in(progress),
    EasingFunction::EaseOut => ease_out(progress),
    EasingFunction::EaseInOutCubic => ease_in_out_cubic(progress),
    EasingFunction::EaseInCubic => ease_in_cubic(progress),
    EasingFunction::EaseOutCubic => ease_out_cubic(progress),
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

/// Quadratic ease-in-out function.
fn ease_in_out(t: f32) -> f32 {
  if t < 0.5 {
    2.0 * t * t
  } else {
    -1.0 + (4.0 - 2.0 * t) * t
  }
}

/// Quadratic ease-in function.
fn ease_in(t: f32) -> f32 {
  t * t
}

/// Quadratic ease-out function.
fn ease_out(t: f32) -> f32 {
  t * (2.0 - t)
}

/// Cubic ease-in-out function.
fn ease_in_out_cubic(t: f32) -> f32 {
  if t < 0.5 {
    4.0 * t * t * t
  } else {
    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
  }
}

/// Cubic ease-in function.
fn ease_in_cubic(t: f32) -> f32 {
  t * t * t
}

/// Cubic ease-out function.
fn ease_out_cubic(t: f32) -> f32 {
  1.0 - (1.0 - t).powi(3)
}

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

