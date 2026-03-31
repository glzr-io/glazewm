use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::NSRect;

use crate::Rect;

impl From<CGRect> for Rect {
  fn from(value: CGRect) -> Self {
    #[allow(clippy::cast_possible_truncation)]
    Rect::from_xy(
      value.origin.x as i32,
      value.origin.y as i32,
      value.size.width as i32,
      value.size.height as i32,
    )
  }
}

impl From<Rect> for CGRect {
  fn from(value: Rect) -> Self {
    CGRect {
      origin: CGPoint {
        x: f64::from(value.x()),
        y: f64::from(value.y()),
      },
      size: CGSize {
        width: f64::from(value.width()),
        height: f64::from(value.height()),
      },
    }
  }
}

pub(crate) trait NSRectExt {
  /// Transforms an AppKit screen rectangle (e.g. `NSScreen.visibleFrame`)
  /// into Core Graphics coordinate space (e.g. `CGDisplayBounds`).
  ///
  /// AppKit has (0,0) at the bottom-left corner of the primary display,
  /// whereas Core Graphics has it at the top-left corner. So we can
  /// convert between the two by offsetting the Y-axis by the primary
  /// display's height.
  fn to_cg_rect(&self, primary_display_bounds: &Rect) -> CGRect;
}

impl NSRectExt for NSRect {
  fn to_cg_rect(&self, primary_display_bounds: &Rect) -> CGRect {
    let adjusted_y = f64::from(primary_display_bounds.height())
      - (self.origin.y + self.size.height);

    CGRect::new(
      CGPoint {
        x: self.origin.x,
        y: adjusted_y,
      },
      CGSize {
        width: self.size.width,
        height: self.size.height,
      },
    )
  }
}
