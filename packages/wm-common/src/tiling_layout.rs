use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TilingLayout {
  #[default]
  Tiles,
  Accordion,
}

impl TilingLayout {
  /// Gets the inverse of a given tiling layout.
  ///
  /// Example:
  /// ```
  /// # use wm_common::TilingLayout;
  /// let layout = TilingLayout::Tiles.inverse();
  /// assert_eq!(layout, TilingLayout::Accordion);
  /// ```
  #[must_use]
  pub fn inverse(&self) -> Self {
    match self {
      Self::Tiles => Self::Accordion,
      Self::Accordion => Self::Tiles,
    }
  }
}

impl FromStr for TilingLayout {
  type Err = anyhow::Error;

  /// Parses a string into a tiling layout.
  ///
  /// Example:
  /// ```
  /// # use std::str::FromStr;
  /// # use wm_common::TilingLayout;
  /// let layout = TilingLayout::from_str("accordion");
  /// assert_eq!(layout.unwrap(), TilingLayout::Accordion);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    match unparsed {
      "tiles" => Ok(Self::Tiles),
      "accordion" => Ok(Self::Accordion),
      _ => bail!("Not a valid tiling layout: {}", unparsed),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn inverse_switches_between_layouts() {
    assert_eq!(TilingLayout::Tiles.inverse(), TilingLayout::Accordion);
    assert_eq!(TilingLayout::Accordion.inverse(), TilingLayout::Tiles);
  }

  #[test]
  fn parses_layout_names() {
    assert_eq!(
      "tiles".parse::<TilingLayout>().unwrap(),
      TilingLayout::Tiles
    );
    assert_eq!(
      "accordion".parse::<TilingLayout>().unwrap(),
      TilingLayout::Accordion
    );
    assert!(TilingLayout::from_str("columns").is_err());
  }

  #[test]
  fn serializes_layout_names_as_snake_case() {
    assert_eq!(
      serde_json::to_string(&TilingLayout::Accordion).unwrap(),
      "\"accordion\""
    );
    assert_eq!(
      serde_json::from_str::<TilingLayout>("\"tiles\"").unwrap(),
      TilingLayout::Tiles
    );
  }
}
