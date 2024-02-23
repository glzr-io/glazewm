pub enum TilingDirection {
  Vertical,
  Horizontal,
}

impl TilingDirection {
  /// Get the inverse of a given tiling direction.
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
}
