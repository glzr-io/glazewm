use std::str::FromStr;

use anyhow::Context;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpacityValue(f32);

impl OpacityValue {
  #[must_use]
  pub fn to_alpha(&self) -> u8 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let alpha = (self.0 * 255.0).round() as u8;
    alpha
  }

  #[must_use]
  pub fn from_alpha(alpha: u8) -> Self {
    Self(f32::from(alpha) / 255.0)
  }
}

impl Default for OpacityValue {
  fn default() -> Self {
    Self(100.0)
  }
}

impl FromStr for OpacityValue {
  type Err = anyhow::Error;

  /// Parses a string for an opacity value. The string must be a percentage
  /// or a decimal number.
  ///
  /// Example:
  /// ```
  /// # use wm::common::{OpacityValue};
  /// # use std::str::FromStr;
  /// let check = OpacityValue(0.75);
  /// let parsed = OpacityValue::from_str("75%");
  /// assert_eq!(parsed.unwrap(), check);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    let unparsed = unparsed.trim();

    if unparsed.ends_with('%') {
      let percentage = unparsed
        .trim_end_matches('%')
        .parse::<f32>()
        .context("Invalid percentage format.")?;

      Ok(Self(percentage / 100.0))
    } else {
      unparsed
        .parse::<f32>()
        .map(Self)
        .context("Invalid decimal format.")
    }
  }
}

/// Deserialize an `OpacityValue` from either a number or a string.
impl<'de> Deserialize<'de> for OpacityValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged, rename_all = "camelCase")]
    enum OpacityValueDe {
      Number(f32),
      String(String),
    }

    match OpacityValueDe::deserialize(deserializer)? {
      OpacityValueDe::Number(num) => Ok(Self(num)),
      OpacityValueDe::String(str) => {
        Self::from_str(&str).map_err(serde::de::Error::custom)
      }
    }
  }
}
