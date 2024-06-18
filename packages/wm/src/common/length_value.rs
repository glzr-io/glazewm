use std::str::FromStr;

use anyhow::{bail, Context};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct LengthValue {
  pub amount: f32,
  pub unit: LengthUnit,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LengthUnit {
  Percentage,
  Pixel,
}

impl LengthValue {
  pub fn from_px(px: i32) -> Self {
    Self {
      amount: px as f32,
      unit: LengthUnit::Pixel,
    }
  }

  pub fn to_px(&self, total_px: i32) -> i32 {
    match self.unit {
      LengthUnit::Percentage => (self.amount * total_px as f32) as i32,
      LengthUnit::Pixel => self.amount as i32,
    }
  }

  pub fn to_percentage(&self, total_px: i32) -> f32 {
    match self.unit {
      LengthUnit::Percentage => self.amount,
      LengthUnit::Pixel => self.amount / total_px as f32,
    }
  }
}

impl FromStr for LengthValue {
  type Err = anyhow::Error;

  /// Parses a string containing a number followed by a unit (`px`, `%`).
  /// Allows for negative numbers.
  ///
  /// Example:
  /// ```
  /// LengthValue::from_str("100px") // { amount: 100.0, unit: LengthUnit::Pixel }
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    let units_regex = Regex::new(r"([+-]?\d+)(%|px)?")?;

    let err_msg = format!(
      "Not a valid length value '{}'. Must be of format '10px' or '10%'.",
      unparsed
    );

    let captures = units_regex
      .captures(unparsed)
      .context(err_msg.to_string())?;

    let unit_str = captures.get(2).map_or("", |m| m.as_str());
    let unit = match unit_str {
      "px" | "" => LengthUnit::Pixel,
      "%" => LengthUnit::Percentage,
      _ => bail!(err_msg),
    };

    let amount = captures
      .get(1)
      .and_then(|amount_str| f32::from_str(amount_str.into()).ok())
      // Store percentage units as a fraction of 1.
      .map(|amount| match unit {
        LengthUnit::Pixel => amount,
        LengthUnit::Percentage => amount / 100.0,
      })
      .context(err_msg.to_string())?;

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
