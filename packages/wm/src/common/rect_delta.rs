use std::ops::Mul;

use serde::{Deserialize, Serialize};

use super::LengthValue;

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct RectDelta {
  /// The delta in x-coordinates on the left of the rectangle.
  pub left: LengthValue,

  /// The delta in y-coordinates on the top of the rectangle.
  pub top: LengthValue,

  /// The delta in x-coordinates on the right of the rectangle.
  pub right: LengthValue,

  /// The delta in y-coordinates on the bottom of the rectangle.
  pub bottom: LengthValue,
}

impl RectDelta {
  pub fn new(
    left: LengthValue,
    top: LengthValue,
    right: LengthValue,
    bottom: LengthValue,
  ) -> Self {
    Self {
      left,
      top,
      right,
      bottom,
    }
  }
}

impl Mul<f32> for RectDelta {
  type Output = Self;
  fn mul(self, rhs: f32) -> Self {
    Self::new(
      self.left * rhs,
      self.top * rhs,
      self.right * rhs,
      self.bottom * rhs,
    )
  }
}
