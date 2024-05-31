use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ColorRGBA {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub a: u8,
}

impl ColorRGBA {
  pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
    Self { r, g, b, a }
  }

  pub fn to_bgr(&self) -> u32 {
    let bgr = format!("{:02x}{:02x}{:02x}", self.b, self.g, self.r);
    u32::from_str_radix(&bgr, 16).unwrap()
  }
}

impl FromStr for ColorRGBA {
  type Err = anyhow::Error;

  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    let mut chars = unparsed.chars();

    if chars.next() != Some('#') {
      bail!("Color must start with a `#`.");
    }

    println!("r: {}", &unparsed[1..3]);
    println!("g: {}", &unparsed[3..5]);
    println!("b: {}", &unparsed[5..7]);
    println!("{}", u8::from_str_radix("ff", 16).unwrap());

    let r = u8::from_str_radix(&unparsed[1..3], 16)?;
    let g = u8::from_str_radix(&unparsed[3..5], 16)?;
    let b = u8::from_str_radix(&unparsed[5..7], 16)?;
    let a = if unparsed.len() == 9 {
      u8::from_str_radix(&unparsed[7..9], 16)?
    } else {
      255
    };

    Ok(Self { r, g, b, a })
  }
}

impl<'de> Deserialize<'de> for ColorRGBA {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let str = String::deserialize(deserializer)?;
    Self::from_str(&str).map_err(serde::de::Error::custom)
  }
}
