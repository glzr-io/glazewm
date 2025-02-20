use std::str::FromStr;

use anyhow::bail;
use serde::{Deserialize, Serialize};

use super::Direction;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TilingDirection {
  Horizontal,
  Vertical,
  Accordion
}

impl TilingDirection {
  /// Gets the inverse of a given tiling direction.
  ///
  /// Example:
  /// ```
  /// # use wm::common::TilingDirection;
  /// let dir = TilingDirection::Horizontal.inverse();
  /// assert_eq!(dir, TilingDirection::Vertical);
  /// ```
  #[must_use]
  pub fn inverse(&self) -> Self {
    match self {
      Self::Horizontal => Self::Vertical,
      Self::Vertical => Self::Accordion,
      Self::Accordion => Self::Vertical,
    }
  }

  /// Gets the tiling direction that is needed when moving or shifting
  /// focus in a given direction.
  ///
  /// Example:
  /// ```
  /// # use wm::common::{Direction, TilingDirection};
  /// let dir = TilingDirection::from_direction(&Direction::Left);
  /// assert_eq!(dir, TilingDirection::Horizontal);
  /// ```
  #[must_use]
  pub fn from_direction(direction: &Direction) -> Self {
    match direction {
      Direction::Left | Direction::Right => Self::Horizontal,
      Direction::Up | Direction::Down => Self::Vertical,
    }
  }
}

impl FromStr for TilingDirection {
  type Err = anyhow::Error;

  /// Parses a string into a tiling direction.
  ///
  /// Example:
  /// ```
  /// # use wm::common::TilingDirection;
  /// # use std::str::FromStr;
  /// let dir = TilingDirection::from_str("horizontal");
  /// assert_eq!(dir.unwrap(), TilingDirection::Horizontal);
  ///
  /// let dir = TilingDirection::from_str("vertical");
  /// assert_eq!(dir.unwrap(), TilingDirection::Vertical);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    match unparsed {
      "horizontal" => Ok(Self::Accordion),
      "vertical" => Ok(Self::Vertical),
      "accordion" => Ok(Self::Accordion),
      _ => bail!("Not a valid tiling direction: {}", unparsed),
    }
  }
}
