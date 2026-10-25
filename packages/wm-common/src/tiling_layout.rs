use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};

/// How the tiling children of a direction container (workspace or split
/// container) are laid out.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TilingLayout {
  /// Children are placed side by side along the tiling direction.
  #[default]
  Tiles,

  /// Children overlap and take up the full container, with the focused
  /// child on top. Neighbors of the focused child peek out by the
  /// configured accordion padding.
  Accordion,
}

impl TilingLayout {
  /// Gets the other layout.
  ///
  /// Example:
  /// ```
  /// # use wm_common::TilingLayout;
  /// let layout = TilingLayout::Tiles.toggled();
  /// assert_eq!(layout, TilingLayout::Accordion);
  /// ```
  #[must_use]
  pub fn toggled(&self) -> Self {
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
  /// # use wm_common::TilingLayout;
  /// # use std::str::FromStr;
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
