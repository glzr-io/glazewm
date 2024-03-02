use std::str::FromStr;

use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone)]
pub struct LengthValue {
  pub amount: f32,
  pub unit: LengthUnit,
}

#[derive(Debug, Clone)]
pub enum LengthUnit {
  Pixel,
  Percentage,
}

impl LengthValue {
  /// Parses a string containing a number followed by a unit (`px`, `%`).
  ///
  /// Example:
  /// ```
  /// LengthValue::from_str("100px") // { amount: 100.0, unit: LengthUnit::Pixel }
  /// ```
  pub fn from_str(unparsed: &str) -> Result<LengthValue> {
    let units_regex = Regex::new(r"(\d+)(%|ppt|px)?")?;

    let captures = units_regex
      .captures(unparsed)
      .context("Invalid length value.")?;

    let amount = f32::from_str(&captures[1])?;

    let unit_str = captures.get(2).map_or("", |m| m.as_str());
    let unit = match unit_str {
      "px" | "" => LengthUnit::Pixel,
      "%" => LengthUnit::Percentage,
      _ => bail!("Not a valid unit '{}'.", unit_str),
    };

    Ok(LengthValue { amount, unit })
  }
}

impl<'de> Deserialize<'de> for LengthValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let str = String::deserialize(deserializer)?;
    Self::from_str(&str).map_err(serde::de::Error::custom)
  }
}
