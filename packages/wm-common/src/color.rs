use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct Color {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,
}

impl Color {
  pub fn to_bgr(&self) -> anyhow::Result<u32> {
    let bgr = format!("{:02x}{:02x}{:02x}", self.b, self.g, self.r);
    Ok(u32::from_str_radix(&bgr, 16)?)
  }
}

impl FromStr for Color {
  type Err = anyhow::Error;

  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    let mut chars = unparsed.chars();

    if chars.next() != Some('#') {
      bail!("Color must start with a `#`.");
    }

    let r = u8::from_str_radix(&unparsed[1..3], 16)?;
    let g = u8::from_str_radix(&unparsed[3..5], 16)?;
    let b = u8::from_str_radix(&unparsed[5..7], 16)?;

    let a = match unparsed.len() {
      9 => u8::from_str_radix(&unparsed[7..9], 16)?,
      7 => 255,
      _ => bail!(
        "Expected color to be either a 6 or 8 character long hex value."
      ),
    };

    Ok(Self { r, g, b, a })
  }
}

/// Deserialize a `Color` from either a string or a struct.
impl<'de> Deserialize<'de> for Color {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ColorDe {
      Struct { r: u8, g: u8, b: u8, a: u8 },
      String(String),
    }

    match ColorDe::deserialize(deserializer)? {
      ColorDe::Struct { r, g, b, a } => Ok(Self { r, g, b, a }),
      ColorDe::String(str) => {
        Self::from_str(&str).map_err(serde::de::Error::custom)
      }
    }
  }
}
