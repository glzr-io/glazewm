use std::str::FromStr;

use anyhow::{bail, Context};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpacityValue {
  pub amount: f32,
  pub unit: OpacityUnit,
  pub is_delta: bool,
  pub delta_sign: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpacityUnit {
  Alpha,
  Percentage,
}

impl OpacityValue {
  pub fn to_exact(&self) -> u8 {
    match self.unit {
      OpacityUnit::Alpha => self.amount as u8,
      OpacityUnit::Percentage => (self.amount * 255.0) as u8,
    }
  }
}

impl Default for OpacityValue {
  fn default() -> Self {
    Self {
      amount: 255.0,
      unit: OpacityUnit::Alpha,
      is_delta: false,
      delta_sign: false,
    }
  }
}

impl FromStr for OpacityValue {
  type Err = anyhow::Error;

  /// Parses a string for an opacity value. The string can be a number
  /// or a percentage. If the string starts with a sign, the value is
  /// interpreted as a delta.
  ///
  /// Example:
  /// ```
  /// # use wm::common::{OpacityValue, OpacityUnit};
  /// # use std::str::FromStr;
  /// let check = OpacityValue {
  ///   amount: 0.75,
  ///   unit: OpacityUnit::Percentage,
  ///   is_delta: false,
  ///   delta_sign: false,
  /// };
  /// let parsed = OpacityValue::from_str("75%");
  /// assert_eq!(parsed.unwrap(), check);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    let units_regex = Regex::new(r"([+-]?)(\d+)(%?)")?;

    let err_msg = format!(
      "Not a valid opacity value '{}'. Must be of format '255', '100%', '+10%' or '-128'.",
      unparsed
    );

    let captures = units_regex
      .captures(unparsed)
      .context(err_msg.to_string())?;

    let sign_str = captures.get(1).map_or("", |m| m.as_str());
    let delta_sign = sign_str == "+";

    // interpret value as a delta if it explicitly starts with a sign
    let is_delta = !sign_str.is_empty();

    let unit_str = captures.get(3).map_or("", |m| m.as_str());
    let unit = match unit_str {
      "" => OpacityUnit::Alpha,
      "%" => OpacityUnit::Percentage,
      _ => bail!(err_msg),
    };

    let amount = captures
      .get(2)
      .and_then(|amount_str| f32::from_str(amount_str.into()).ok())
      // Store percentage units as a fraction of 1.
      .map(|amount| match unit {
        OpacityUnit::Alpha => amount,
        OpacityUnit::Percentage => amount / 100.0,
      })
      .context(err_msg.to_string())?;

    Ok(OpacityValue {
      amount,
      unit,
      is_delta,
      delta_sign,
    })
  }
}

/// Deserialize an `OpacityValue` from either a string or a struct.
impl<'de> Deserialize<'de> for OpacityValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged, rename_all = "camelCase")]
    enum OpacityValueDe {
      Struct {
        amount: f32,
        unit: OpacityUnit,
        is_delta: bool,
        delta_sign: bool,
      },
      String(String),
    }

    match OpacityValueDe::deserialize(deserializer)? {
      OpacityValueDe::Struct {
        amount,
        unit,
        is_delta,
        delta_sign,
      } => Ok(Self {
        amount,
        unit,
        is_delta,
        delta_sign,
      }),
      OpacityValueDe::String(str) => {
        Self::from_str(&str).map_err(serde::de::Error::custom)
      }
    }
  }
}
