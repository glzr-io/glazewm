pub struct RectDelta {
  /// The diff in x-coordinates of the upper-left corner of the rectangle.
  left: i32,

  /// The diff in y-coordinates of the upper-left corner of the rectangle.
  top: i32,

  /// The diff in x-coordinates of the lower-right corner of the rectangle.
  right: i32,

  /// The diff in y-coordinates of the lower-right corner of the rectangle.
  bottom: i32,
}

impl RectDelta {
  pub fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
    Self {
      left,
      top,
      right,
      bottom,
    }
  }
}
