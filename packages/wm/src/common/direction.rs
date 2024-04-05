use std::str::FromStr;

use anyhow::bail;

#[derive(Clone, Debug)]
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
  /// Direction::Left.inverse() // Direction::Right
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
  /// Direction::from_str("left") // Direction::Left
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
