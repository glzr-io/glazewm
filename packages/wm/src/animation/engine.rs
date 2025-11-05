use std::time::{Duration, Instant};
use wm_common::{EasingFunction, OpacityValue, Rect};

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

/// Interpolates between two rectangles.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn interpolate_rect(start: &Rect, end: &Rect, progress: f32) -> Rect {
  let x = start.x() as f32 + (end.x() - start.x()) as f32 * progress;
  let y = start.y() as f32 + (end.y() - start.y()) as f32 * progress;
  let width = start.width() as f32 + (end.width() - start.width()) as f32 * progress;
  let height = start.height() as f32 + (end.height() - start.height()) as f32 * progress;

  Rect::from_xy(
    x.round() as i32,
    y.round() as i32,
    width.round() as i32,
    height.round() as i32,
  )
}

/// Interpolates between two opacity values.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss)]
pub fn interpolate_opacity(
  start: &OpacityValue,
  end: &OpacityValue,
  progress: f32,
) -> OpacityValue {
  let start_alpha = start.to_alpha() as f32;
  let end_alpha = end.to_alpha() as f32;
  let alpha = start_alpha + (end_alpha - start_alpha) * progress;

  OpacityValue::from_alpha((alpha.round() as u32).clamp(0, 255) as u8)
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

/// Scales a rectangle from its center point.
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn scale_rect_from_center(rect: &Rect, scale: f32) -> Rect {
  let center_x = rect.x() + rect.width() / 2;
  let center_y = rect.y() + rect.height() / 2;

  let new_width = (rect.width() as f32 * scale).round() as i32;
  let new_height = (rect.height() as f32 * scale).round() as i32;

  let new_x = center_x - new_width / 2;
  let new_y = center_y - new_height / 2;

  Rect::from_xy(new_x, new_y, new_width, new_height)
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

    let mid = interpolate_rect(&start, &end, 0.5);
    assert_eq!(mid.x(), 50);
    assert_eq!(mid.y(), 50);
    assert_eq!(mid.width(), 150);
    assert_eq!(mid.height(), 150);
  }

  #[test]
  fn test_scale_rect_from_center() {
    let rect = Rect::from_xy(100, 100, 200, 200);
    let scaled = scale_rect_from_center(&rect, 0.5);

    assert_eq!(scaled.width(), 100);
    assert_eq!(scaled.height(), 100);
    assert_eq!(scaled.x(), 150);
    assert_eq!(scaled.y(), 150);
  }
}

