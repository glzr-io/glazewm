use crate::{Color, Dispatcher, NativeWindow, Rect, WindowId};

/// A reusable overlay border for visually highlighting the focused window.
///
/// # Platform-specific
///
/// This type is only available on Windows.
pub struct FocusedWindowBorder {
  inner: crate::platform_impl::FocusedWindowBorder,
}

impl FocusedWindowBorder {
  /// Creates a focused-window border overlay.
  pub fn new(dispatcher: Dispatcher) -> crate::Result<Self> {
    Ok(Self {
      inner: crate::platform_impl::FocusedWindowBorder::new(dispatcher)?,
    })
  }

  /// Shows the border around the given window.
  pub fn show(
    &mut self,
    tracked_window: &NativeWindow,
    frame: &Rect,
    color: &Color,
  ) -> crate::Result<()> {
    self.inner.show(tracked_window.id(), frame, color)
  }

  /// Updates the border position for the currently tracked window.
  pub fn update_position(
    &mut self,
    tracked_window: &NativeWindow,
    frame: &Rect,
    color: &Color,
  ) -> crate::Result<()> {
    self
      .inner
      .update_position(tracked_window.id(), frame, color)
  }

  /// Hides the border overlay.
  pub fn hide(&mut self) -> crate::Result<()> {
    self.inner.hide()
  }

  /// Destroys the border overlay window.
  pub fn shutdown(&mut self) -> crate::Result<()> {
    self.inner.shutdown()
  }

  /// Gets the currently tracked window ID.
  #[must_use]
  pub fn tracked_window_id(&self) -> Option<WindowId> {
    self.inner.tracked_window_id()
  }
}
