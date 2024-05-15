use super::RectDelta;

#[derive(Debug, Clone)]
pub struct Rect {
  /// X-coordinate of the left edge of the rectangle.
  pub left: i32,

  /// Y-coordinate of the top edge of the rectangle.
  pub top: i32,

  /// X-coordinate of the right edge of the rectangle.
  pub right: i32,

  /// Y-coordinate of the bottom edge of the rectangle.
  pub bottom: i32,
}

impl Rect {
  /// Creates a new `Rect` instance from the coordinates of its left, top,
  /// right, and bottom edges.
  pub fn from_ltrb(left: i32, top: i32, right: i32, bottom: i32) -> Self {
    Self {
      left,
      top,
      right,
      bottom,
    }
  }

  /// Creates a new `Rect` instance from its X/Y coordinates and size.
  pub fn from_xy(x: i32, y: i32, width: i32, height: i32) -> Self {
    Self {
      left: x,
      top: y,
      right: x + width,
      bottom: y + height,
    }
  }

  pub fn x(&self) -> i32 {
    self.left
  }

  pub fn y(&self) -> i32 {
    self.top
  }

  pub fn width(&self) -> i32 {
    self.right - self.left
  }

  pub fn height(&self) -> i32 {
    self.bottom - self.top
  }

  pub fn translate_to_coordinates(&mut self, x: i32, y: i32) -> Self {
    Self::from_xy(x, y, self.width(), self.height())
  }

  pub fn translate_to_center(&mut self, outer_rect: &Rect) -> Self {
    Self::translate_to_coordinates(
      self,
      outer_rect.left + (outer_rect.width() / 2) - (self.width() / 2),
      outer_rect.top + (outer_rect.height() / 2) - (self.height() / 2),
    )
  }

  pub fn center_point(&self) -> (i32, i32) {
    (
      self.left + (self.width() / 2),
      self.top + (self.height() / 2),
    )
  }

  pub fn apply_delta(&self, delta: &RectDelta) -> Self {
    Self::from_ltrb(
      self.left - delta.left.to_pixels(self.width()),
      self.top - delta.top.to_pixels(self.height()),
      self.right + delta.right.to_pixels(self.width()),
      self.bottom + delta.bottom.to_pixels(self.height()),
    )
  }

  // Gets whether the x-coordinate overlaps with the x-coordinate of the
  // other rect.
  pub fn has_overlap_x(&self, other: &Rect) -> bool {
    !(self.x() + self.width() <= other.x()
      || other.x() + other.width() <= self.x())
  }

  // Gets whether the y-coordinate overlaps with the y-coordinate of the
  // other rect.
  pub fn has_overlap_y(&self, other: &Rect) -> bool {
    !(self.y() + self.height() <= other.y()
      || other.y() + other.height() <= self.y())
  }
}
