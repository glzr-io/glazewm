pub enum Direction {
  Left,
  Right,
  Up,
  Down,
}

impl Direction {
  /// Get the inverse of a given direction.
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

  /// Get the tiling direction that is needed when moving or switching
  /// focus in given direction.
  ///
  /// Example:
  /// ```
  /// Direction::Left.to_tiling_direction() // TilingDirection::Horizontal
  /// ```
  pub fn to_tiling_direction(&self) -> TilingDirection {
    match self {
      Direction::Left | Direction::Right => TilingDirection.Horizontal,
      Direction::Up | Direction::Down => TilingDirection.Vertical,
    }
  }
}
