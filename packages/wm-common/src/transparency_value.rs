use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TransparencyValue(f32);

impl TransparencyValue {
  #[must_use]
  pub fn to_alpha(&self) -> i16 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let alpha = (self.0 / 100.0 * 255.0).round() as i16;
    alpha
  }
}

impl Default for TransparencyValue {
  fn default() -> Self {
    Self(100.0)
  }
}

impl FromStr for TransparencyValue {
  type Err = anyhow::Error;

  /// Parses a string for a transparency value. The string must be
  /// a percentage or a decimal number.
  ///
  /// Example:
  /// ```
  /// # use wm::common::{TransparencyValue};
  /// # use std::str::FromStr;
  /// let check = TransparencyValue(0.75);
  /// let parsed = TransparencyValue::from_str("75%");
  /// assert_eq!(parsed.unwrap(), check);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    let s = unparsed.trim();

    if s.ends_with('%') {
      let percentage = s
        .trim_end_matches('%')
        .parse::<f32>()
        .map_err(|_| anyhow::anyhow!("Invalid percentage format"))?;
      Ok(Self(percentage / 100.0))
    } else {
      s.parse::<f32>()
        .map(Self)
        .map_err(|_| anyhow::anyhow!("Invalid decimal format"))
    }
  }
}

/// Deserialize an `TransparencyValue` from either a number or a string.
impl<'de> Deserialize<'de> for TransparencyValue {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged, rename_all = "camelCase")]
    enum TransparencyValueDe {
      Number(f32),
      String(String),
    }

    match TransparencyValueDe::deserialize(deserializer)? {
      TransparencyValueDe::Number(num) => Ok(Self(num)),
      TransparencyValueDe::String(str) => {
        Self::from_str(&str).map_err(serde::de::Error::custom)
      }
    }
  }
}
