use std::str::FromStr;

use anyhow::Context;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TransparencyValue {
  pub percent: u32,
}

impl TransparencyValue {
  pub fn to_alpha(self) -> u32 {
    self.percent / 100 * 255
  }
}

impl Default for TransparencyValue {
  fn default() -> Self {
    Self { percent: 100 }
  }
}

impl FromStr for TransparencyValue {
  type Err = anyhow::Error;

  /// Parses a string for a transparency value. The string must be
  /// percentage and can either be positive or negative.
  ///
  /// Example:
  /// ```
  /// # use wm::common::{TransparencyValue};
  /// # use std::str::FromStr;
  /// let check = TransparencyValue {
  ///   percent: 70,
  /// };
  /// let parsed = TransparencyValue::from_str("75%");
  /// assert_eq!(parsed.unwrap(), check);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    let units_regex = Regex::new(r"([+-]?\d+)%")?;

    let err_msg = format!(
      "Not a valid transparency value '{unparsed}'. Must be formatted as percentage, e.g. '70%'."
    );

    let captures = units_regex
      .captures(unparsed)
      .context(err_msg.to_string())?;

    let unit_str = captures.get(3).map_or("", |m| m.as_str());

    #[allow(clippy::cast_possible_truncation)]
    let percent = captures
      .get(2)
      .and_then(|amount_str| f32::from_str(amount_str.into()).ok())
      // Convert percentages to 0-255 range.
      .map(|amount| match unit_str {
        "%" => (amount / 100.0 * 255.0).round() as f32,
        _ => amount.round() as f32,
      })
      // Negate the value if it's a negative delta.
      // Since an explicit sign tells us it's a delta,
      // a negative Alpha value is impossible.
      .map(|amount| if sign_str == "-" { -amount } else { amount })
      .context(err_msg.to_string())?;

    Ok(TransparencyValue { percent })
  }
}

/// Deserialize an `OpacityValue` from either a string or a struct.
impl<'de> Deserialize<'de> for TransparencyValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged, rename_all = "camelCase")]
    enum TransparencyValueDe {
      Struct { percent: u32 },
      String(String),
    }

    match TransparencyValueDe::deserialize(deserializer)? {
      TransparencyValueDe::Struct { percent } => Ok(Self { percent }),
      TransparencyValueDe::String(str) => {
        Self::from_str(&str).map_err(serde::de::Error::custom)
      }
    }
  }
}
