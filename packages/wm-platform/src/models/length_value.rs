use std::str::FromStr;

use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LengthValue {
  pub amount: f32,
  pub unit: LengthUnit,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LengthUnit {
  Percentage,
  Pixel,
}

impl LengthValue {
  #[must_use]
  pub fn from_px(px: i32) -> Self {
    Self {
      #[allow(clippy::cast_precision_loss)]
      amount: px as f32,
      unit: LengthUnit::Pixel,
    }
  }

  #[must_use]
  pub fn to_px(&self, total_px: i32, scale_factor: Option<f32>) -> i32 {
    let scale_factor = scale_factor.unwrap_or(1.0);

    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    match self.unit {
      LengthUnit::Percentage => (self.amount * total_px as f32) as i32,
      LengthUnit::Pixel => (self.amount * scale_factor) as i32,
    }
  }

  #[must_use]
  pub fn to_percentage(&self, total_px: i32) -> f32 {
    match self.unit {
      LengthUnit::Percentage => self.amount,
      #[allow(clippy::cast_precision_loss)]
      LengthUnit::Pixel => self.amount / total_px as f32,
    }
  }
}

impl FromStr for LengthValue {
  type Err = crate::ParseError;

  /// Parses a string containing a number followed by a unit (`px`, `%`).
  /// Allows for negative numbers.
  ///
  /// Example:
  /// ```
  /// # use wm_platform::{LengthValue, LengthUnit};
  /// # use std::str::FromStr;
  /// let check = LengthValue {
  ///   amount: 100.0,
  ///   unit: LengthUnit::Pixel,
  /// };
  /// let parsed = LengthValue::from_str("100px");
  /// assert_eq!(parsed.unwrap(), check);
  /// ```
  fn from_str(unparsed: &str) -> Result<Self, crate::ParseError> {
    let units_regex =
      Regex::new(r"([+-]?\d+)(%|px)?").expect("Invalid regex.");

    let captures = units_regex
      .captures(unparsed)
      .ok_or(crate::ParseError::Length(unparsed.to_string()))?;

    let unit = match captures.get(2).map_or("", |m| m.as_str()) {
      "px" | "" => LengthUnit::Pixel,
      "%" => LengthUnit::Percentage,
      _ => return Err(crate::ParseError::Length(unparsed.to_string())),
    };

    let amount = captures
      .get(1)
      .and_then(|m| m.as_str().parse::<f32>().ok())
      // Store percentage units as a fraction of 1.
      .map(|amount| {
        if unit == LengthUnit::Percentage {
          amount / 100.0
        } else {
          amount
        }
      })
      .ok_or(crate::ParseError::Length(unparsed.to_string()))?;

    Ok(LengthValue { amount, unit })
  }
}

/// Deserialize a `LengthValue` from either a string or a struct.
impl<'de> Deserialize<'de> for LengthValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum LengthValueDe {
      Struct { amount: f32, unit: LengthUnit },
      String(String),
    }

    match LengthValueDe::deserialize(deserializer)? {
      LengthValueDe::Struct { amount, unit } => Ok(Self { amount, unit }),
      LengthValueDe::String(str) => {
        Self::from_str(&str).map_err(serde::de::Error::custom)
      }
    }
  }
}
