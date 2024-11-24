use std::str::FromStr;

use anyhow::{bail, Context};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TransparencyValue {
  pub amount: f32,
  pub unit: TransparencyUnit,
  pub is_delta: bool,
  pub delta_sign: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransparencyUnit {
  Exact,
  Percentage,
}

impl TransparencyValue {
  pub fn to_exact(&self) -> u8 {
    match self.unit {
      TransparencyUnit::Exact => self.amount as u8,
      TransparencyUnit::Percentage => (self.amount * 255.0) as u8,
    }
  }
}

impl Default for TransparencyValue {
  fn default() -> Self {
    Self {
      amount: 1.0,
      unit: TransparencyUnit::Exact,
      is_delta: false,
      delta_sign: false,
    }
  }
}

impl FromStr for TransparencyValue {
  type Err = anyhow::Error;

  /// Parses a string containing a number possibly followed by a percentage
  /// sign. Allows for negative numbers.
  ///
  /// Example:
  /// ```
  /// # use wm::common::{TransparencyValue, TransparencyUnit};
  /// # use std::str::FromStr;
  /// let check = TransparencyValue {
  ///   amount: 0.75,
  ///   unit: TransparencyUnit::Percentage,
  /// };
  /// let parsed = TransparencyValue::from_str("75%");
  /// assert_eq!(parsed.unwrap(), check);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    let units_regex = Regex::new(r"([+-]?)(\d+)(%?)")?;

    let err_msg = format!(
      "Not a valid transparency value '{}'. Must be of format '255', '100%', '+10%' or '-128'.",
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
      "" => TransparencyUnit::Exact,
      "%" => TransparencyUnit::Percentage,
      _ => bail!(err_msg),
    };

    let amount = captures
      .get(2)
      .and_then(|amount_str| f32::from_str(amount_str.into()).ok())
      // Store percentage units as a fraction of 1.
      .map(|amount| match unit {
        TransparencyUnit::Exact => amount,
        TransparencyUnit::Percentage => amount / 100.0,
      })
      .context(err_msg.to_string())?;

    Ok(TransparencyValue {
      amount,
      unit,
      is_delta,
      delta_sign,
    })
  }
}

/// Deserialize a `TransparencyValue` from either a string or a struct.
impl<'de> Deserialize<'de> for TransparencyValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged, rename_all = "camelCase")]
    enum TransparencyValueDe {
      Struct {
        amount: f32,
        unit: TransparencyUnit,
        is_delta: bool,
        delta_sign: bool,
      },
      String(String),
    }

    match TransparencyValueDe::deserialize(deserializer)? {
      TransparencyValueDe::Struct {
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
      TransparencyValueDe::String(str) => {
        Self::from_str(&str).map_err(serde::de::Error::custom)
      }
    }
  }
}
