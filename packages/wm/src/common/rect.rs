use serde::{Deserialize, Serialize};

use super::{Direction, Point, RectDelta};

#[derive(Debug, Deserialize, Clone, Serialize, Eq, PartialEq)]
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

  pub fn translate_to_coordinates(&self, x: i32, y: i32) -> Self {
    Self::from_xy(x, y, self.width(), self.height())
  }

  pub fn translate_to_center(&self, outer_rect: &Rect) -> Self {
    Self::translate_to_coordinates(
      self,
      outer_rect.left + (outer_rect.width() / 2) - (self.width() / 2),
      outer_rect.top + (outer_rect.height() / 2) - (self.height() / 2),
    )
  }

  pub fn translate_in_direction(
    &self,
    direction: &Direction,
    distance: i32,
  ) -> Rect {
    let (delta_x, delta_y) = match direction {
      Direction::Up => (0, -distance),
      Direction::Down => (0, distance),
      Direction::Left => (-distance, 0),
      Direction::Right => (distance, 0),
    };

    Self::from_xy(
      self.x() + delta_x,
      self.y() + delta_y,
      self.width(),
      self.height(),
    )
  }

  /// Returns a new `Rect` that is clamped within the bounds of the given
  /// outer rectangle. Attempts to preserve the width and height of the
  /// original rectangle.
  pub fn clamp(&self, outer_rect: &Rect) -> Self {
    Self::from_xy(
      self.left.max(outer_rect.left),
      self.top.max(outer_rect.top),
      self.width().min(outer_rect.width()),
      self.height().min(outer_rect.height()),
    )
  }

  pub fn clamp_size(&self, width: i32, height: i32) -> Self {
    Self::from_xy(
      self.x(),
      self.y(),
      self.width().min(width),
      self.height().min(height),
    )
  }

  pub fn center_point(&self) -> Point {
    Point {
      x: self.left + (self.width() / 2),
      y: self.top + (self.height() / 2),
    }
  }

  // TODO: Pass in a `Rect` for unit conversion.
  pub fn apply_delta(&self, delta: &RectDelta) -> Self {
    Self::from_ltrb(
      self.left - delta.left.to_px(self.width()),
      self.top - delta.top.to_px(self.height()),
      self.right + delta.right.to_px(self.width()),
      self.bottom + delta.bottom.to_px(self.height()),
    )
  }

  // TODO: Pass in a `Rect` for unit conversion.
  pub fn apply_inverse_delta(&self, delta: &RectDelta) -> Self {
    Self::from_ltrb(
      self.left + delta.left.to_px(self.width()),
      self.top + delta.top.to_px(self.height()),
      self.right - delta.right.to_px(self.width()),
      self.bottom - delta.bottom.to_px(self.height()),
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

  pub fn contains_point(&self, point: &Point) -> bool {
    let is_in_x = point.x >= self.left && point.x <= self.right;
    let is_in_y = point.y >= self.top && point.y <= self.bottom;
    return is_in_x && is_in_y;
  }
}
