use std::str::FromStr;

use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
  Left,
  Right,
  Up,
  Down,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResizeSide {
  Left,
  Right,
  Top,
  Bottom,
}

impl Direction {
  /// Gets the inverse of a given direction.
  ///
  /// Example:
  /// ```
  /// # use wm_platform::Direction;
  /// let dir = Direction::Left.inverse();
  /// assert_eq!(dir, Direction::Right);
  /// ```
  #[must_use]
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
  type Err = crate::ParseError;

  /// Parses a string into a direction.
  ///
  /// Example:
  /// ```
  /// # use wm_platform::Direction;
  /// # use std::str::FromStr;
  /// let dir = Direction::from_str("left");
  /// assert_eq!(dir.unwrap(), Direction::Left);
  /// ```
  fn from_str(unparsed: &str) -> Result<Self, crate::ParseError> {
    match unparsed {
      "left" => Ok(Direction::Left),
      "right" => Ok(Direction::Right),
      "up" => Ok(Direction::Up),
      "down" => Ok(Direction::Down),
      _ => Err(crate::ParseError::Direction(unparsed.to_string())),
    }
  }
}

impl FromStr for ResizeSide {
  type Err = crate::ParseError;

  fn from_str(unparsed: &str) -> Result<Self, crate::ParseError> {
    match unparsed {
      "left" => Ok(ResizeSide::Left),
      "right" => Ok(ResizeSide::Right),
      "top" => Ok(ResizeSide::Top),
      "bottom" => Ok(ResizeSide::Bottom),
      _ => Err(crate::ParseError::Direction(unparsed.to_string())),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::ResizeSide;

  #[test]
  fn parses_resize_side() {
    assert_eq!(ResizeSide::from_str("left").unwrap(), ResizeSide::Left);
    assert_eq!(ResizeSide::from_str("right").unwrap(), ResizeSide::Right);
    assert_eq!(ResizeSide::from_str("top").unwrap(), ResizeSide::Top);
    assert_eq!(ResizeSide::from_str("bottom").unwrap(), ResizeSide::Bottom);
  }

  #[test]
  fn rejects_direction_names_that_are_not_resize_sides() {
    assert!(ResizeSide::from_str("up").is_err());
    assert!(ResizeSide::from_str("down").is_err());
  }

  #[test]
  fn rejects_empty_resize_side() {
    assert!(ResizeSide::from_str("").is_err());
  }
}
