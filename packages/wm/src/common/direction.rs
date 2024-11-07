use std::str::FromStr;

use anyhow::bail;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
  Left,
  Right,
  Up,
  Down,
}

impl Direction {
  /// Gets the inverse of a given direction.
  ///
  /// Example:
  /// ```
  /// # use wm::common::Direction;
  /// let dir = Direction::Left.inverse();
  /// assert_eq!(dir, Direction::Right);
  /// ```
  pub fn inverse(&self) -> Direction {
    match self {
      Direction::Left => Direction::Right,
      Direction::Right => Direction::Left,
      Direction::Up => Direction::Down,
      Direction::Down => Direction::Up,
    }
  }
}

impl FromStr for Direction {
  type Err = anyhow::Error;

  /// Parses a string into a direction.
  ///
  /// Example:
  /// ```
  /// # use wm::common::Direction;
  /// # use std::str::FromStr;
  /// let dir = Direction::from_str("left");
  /// assert_eq!(dir.unwrap(), Direction::Left);
  /// ```
  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    match unparsed {
      "left" => Ok(Direction::Left),
      "right" => Ok(Direction::Right),
      "up" => Ok(Direction::Up),
      "down" => Ok(Direction::Down),
      _ => bail!("Not a valid direction: {}", unparsed),
    }
  }
}
