/// Represents an x-y coordinate.
#[derive(Debug, Clone)]
pub struct Point {
  pub x: i32,
  pub y: i32,
}

impl Point {
  /// Calculates the Euclidean distance between this point and another
  /// point.
  #[must_use]
  pub fn distance_between(&self, other: &Point) -> f32 {
    let dx = self.x - other.x;
    let dy = self.y - other.y;

    #[allow(clippy::cast_precision_loss)]
    ((dx * dx + dy * dy) as f32).sqrt()
  }
}
