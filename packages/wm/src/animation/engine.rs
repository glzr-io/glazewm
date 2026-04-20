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
  }
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


/// Interpolates a value with an easing function applied.
pub fn interpolate_with_easing<T>(
  start: &T,
  end: &T,
  progress: f32,
  easing: &EasingFunction,
  interpolate_fn: impl Fn(&T, &T, f32) -> T,
) -> T {
  let eased_progress = apply_easing(progress, easing);
  interpolate_fn(start, end, eased_progress)
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

