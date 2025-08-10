use wm_common::Rect;

use crate::platform_impl;

#[derive(Clone, Debug, PartialEq)]
pub enum ZOrder {
  Normal,
  AfterWindow(isize),
  Top,
  TopMost,
}

#[derive(Clone, Debug)]
pub struct NativeWindow {
  pub(crate) inner: platform_impl::NativeWindow,
}

impl NativeWindow {
  /// Gets the window's title. If the window is invalid, returns an empty
  /// string.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn title(&self) -> crate::Result<String> {
    self.inner.title()
  }

  /// Updates the cached window title.
  pub fn invalidate_title(&self) -> crate::Result<String> {
    self.inner.invalidate_title()
  }

  /// Gets the process name associated with the window.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn process_name(&self) -> crate::Result<String> {
    self.inner.process_name()
  }

  /// Gets the class name of the window.
  ///
  /// This value is lazily retrieved and cached after first retrieval.
  pub fn class_name(&self) -> crate::Result<String> {
    self.inner.class_name()
  }

  /// Whether the window is actually visible.
  pub fn is_visible(&self) -> crate::Result<bool> {
    self.inner.is_visible()
  }

  /// Resize the window to the specified size.
  pub fn resize(&self, size: Rect) -> crate::Result<()> {
    self.inner.resize(size)
  }

  pub fn cleanup(&self) {
    self.inner.cleanup()
  }
}

impl PartialEq for NativeWindow {
  fn eq(&self, other: &Self) -> bool {
    self.inner.handle == other.inner.handle
  }
}

impl Eq for NativeWindow {}
