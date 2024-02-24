use anyhow::{bail, Result};

use super::LengthValue;

#[derive(Debug)]
pub struct RectDelta {
  /// The delta in x-coordinates on the left of the rectangle.
  left: LengthValue,

  /// The delta in y-coordinates on the top of the rectangle.
  top: LengthValue,

  /// The delta in x-coordinates on the right of the rectangle.
  right: LengthValue,

  /// The delta in y-coordinates on the bottom of the rectangle.
  bottom: LengthValue,
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

  pub fn from_str(unparsed: &str) -> Result<Self> {
    let mut parts = unparsed.split_whitespace();

    match parts.count() {
      1 => {
        let value = LengthValue::from_str(parts.next().unwrap())?;
        Ok(Self::new(value, value, value, value))
      }
      2 => {
        let top_bottom = LengthValue::from_str(parts.next().unwrap())?;
        let left_right = LengthValue::from_str(parts.next().unwrap())?;
        Ok(Self::new(left_right, top_bottom, left_right, top_bottom))
      }
      3 => {
        let top = LengthValue::from_str(parts.next().unwrap())?;
        let left_right = LengthValue::from_str(parts.next().unwrap())?;
        let bottom = LengthValue::from_str(parts.next().unwrap())?;
        Ok(Self::new(left_right, top, left_right, bottom))
      }
      4 => {
        let top = LengthValue::from_str(parts.next().unwrap())?;
        let right = LengthValue::from_str(parts.next().unwrap())?;
        let bottom = LengthValue::from_str(parts.next().unwrap())?;
        let left = LengthValue::from_str(parts.next().unwrap())?;
        Ok(Self::new(left, top, right, bottom))
      }
      _ => bail!("Invalid shorthand."),
    }
  }
}
