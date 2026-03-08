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
  #[must_use]
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

  /// Checks if the rectangle delta has a value greater than 1.0(px/%) for
  /// any of its sides.
  #[must_use]
  pub fn is_significant(&self) -> bool {
    self.bottom.amount > 1.0
      || self.top.amount > 1.0
      || self.left.amount > 1.0
      || self.right.amount > 1.0
  }

  /// Creates a new `RectDelta` with all sides set to 0px.
  #[must_use]
  pub fn zero() -> Self {
    Self::new(
      LengthValue::from_px(0),
      LengthValue::from_px(0),
      LengthValue::from_px(0),
      LengthValue::from_px(0),
    )
  }

  /// Gets the inverse of this delta by negating all values.
  ///
  /// Returns a new `RectDelta` instance.
  #[must_use]
  pub fn inverse(&self) -> Self {
    RectDelta::new(
      LengthValue {
        amount: -self.left.amount,
        unit: self.left.unit.clone(),
      },
      LengthValue {
        amount: -self.top.amount,
        unit: self.top.unit.clone(),
      },
      LengthValue {
        amount: -self.right.amount,
        unit: self.right.unit.clone(),
      },
      LengthValue {
        amount: -self.bottom.amount,
        unit: self.bottom.unit.clone(),
      },
    )
  }
}
