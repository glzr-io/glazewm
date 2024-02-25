use anyhow::{bail, Result};

use super::LengthValue;

#[derive(Debug, Clone)]
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
    let parts: Vec<&str> = unparsed.split_whitespace().collect();

    match parts.len() {
      1 => {
        let value = LengthValue::from_str(parts[0])?;
        Ok(Self::new(
          value.clone(),
          value.clone(),
          value.clone(),
          value,
        ))
      }
      2 => {
        let top_bottom = LengthValue::from_str(parts[0])?;
        let left_right = LengthValue::from_str(parts[1])?;
        Ok(Self::new(
          left_right.clone(),
          top_bottom.clone(),
          left_right,
          top_bottom,
        ))
      }
      3 => {
        let top = LengthValue::from_str(parts[0])?;
        let left_right = LengthValue::from_str(parts[1])?;
        let bottom = LengthValue::from_str(parts[2])?;
        Ok(Self::new(left_right.clone(), top, left_right, bottom))
      }
      4 => {
        let top = LengthValue::from_str(parts[0])?;
        let right = LengthValue::from_str(parts[1])?;
        let bottom = LengthValue::from_str(parts[2])?;
        let left = LengthValue::from_str(parts[3])?;
        Ok(Self::new(left, top, right, bottom))
      }
      _ => bail!("Invalid shorthand."),
    }
  }
}
