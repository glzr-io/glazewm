use std::str::FromStr;

use anyhow::bail;
use serde::Serialize;

use super::Direction;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum TilingDirection {
  Vertical,
  Horizontal,
}

impl TilingDirection {
  /// Gets the inverse of a given tiling direction.
  ///
  /// Example:
  /// ```
  /// TilingDirection::Horizontal.inverse() // TilingDirection::Vertical
  /// ```
  pub fn inverse(&self) -> TilingDirection {
    match self {
      TilingDirection::Horizontal => TilingDirection::Vertical,
      TilingDirection::Vertical => TilingDirection::Horizontal,
    }
  }

  /// Gets the tiling direction that is needed when moving or shifting
  /// focus in a given direction.
  ///
  /// Example:
  /// ```
  /// TilingDirection::from_direction(Direction::Left) // TilingDirection::Horizontal
  /// ```
  pub fn from_direction(direction: &Direction) -> TilingDirection {
    match direction {
      Direction::Left | Direction::Right => TilingDirection::Horizontal,
      Direction::Up | Direction::Down => TilingDirection::Vertical,
    }
  }
}

impl FromStr for TilingDirection {
  type Err = anyhow::Error;

  /// Parses a string into a tiling direction.
  ///
  /// Example:
  /// ```
  /// TilingDirection::from_str("horizontal") // TilingDirection::Horizontal
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    match unparsed {
      "horizontal" => Ok(TilingDirection::Horizontal),
      "vertical" => Ok(TilingDirection::Vertical),
      _ => bail!("Not a valid tiling direction: {}", unparsed),
    }
  }
}
