use std::str::FromStr;

use anyhow::Context;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpacityValue {
  pub amount: i16,
  pub is_delta: bool,
}

impl Default for OpacityValue {
  fn default() -> Self {
    Self {
      amount: 255,
      is_delta: false,
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
  /// # use wm::common::{OpacityValue};
  /// # use std::str::FromStr;
  /// let check = OpacityValue {
  ///   amount: 191,
  ///   is_delta: false,
  /// };
  /// let parsed = OpacityValue::from_str("75%");
  /// assert_eq!(parsed.unwrap(), check);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    let units_regex = Regex::new(r"([+-]?)(\d+)(%?)")?;

    let err_msg = format!(
      "Not a valid opacity value '{unparsed}'. Must be of format '255', '100%', '+10%' or '-128'."
    );

    let captures = units_regex
      .captures(unparsed)
      .context(err_msg.to_string())?;

    let sign_str = captures.get(1).map_or("", |m| m.as_str());

    // Interpret value as a delta if it explicitly starts with a sign.
    let is_delta = !sign_str.is_empty();

    let unit_str = captures.get(3).map_or("", |m| m.as_str());

    #[allow(clippy::cast_possible_truncation)]
    let amount = captures
      .get(2)
      .and_then(|amount_str| f32::from_str(amount_str.into()).ok())
      // Convert percentages to 0-255 range.
      .map(|amount| match unit_str {
        "%" => (amount / 100.0 * 255.0).round() as i16,
        _ => amount.round() as i16,
      })
      // Negate the value if it's a negative delta.
      // Since an explicit sign tells us it's a delta,
      // a negative Alpha value is impossible.
      .map(|amount| if sign_str == "-" { -amount } else { amount })
      .context(err_msg.to_string())?;

    Ok(OpacityValue { amount, is_delta })
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
      Struct { amount: f32, is_delta: bool },
      String(String),
    }

    match OpacityValueDe::deserialize(deserializer)? {
      OpacityValueDe::Struct { amount, is_delta } => Ok(Self {
        #[allow(clippy::cast_possible_truncation)]
        amount: amount as i16,
        is_delta,
      }),

      OpacityValueDe::String(str) => {
        Self::from_str(&str).map_err(serde::de::Error::custom)
      }
    }
  }
}
