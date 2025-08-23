use wm_common::Rect;

use crate::platform_impl;

/// Unique identifier of a window.
///
/// Can be obtained with `window.id()`.
///
/// # Platform-specific
///
/// - **Windows**: `isize` (`HWND`)
/// - **macOS**: `u32` (`CGWindowID`)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(
  #[cfg(target_os = "windows")] pub(crate) isize,
  #[cfg(target_os = "macos")] pub(crate) u32,
);

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
  pub fn title(&self) -> crate::Result<String> {
    self.inner.title()
  }

  /// Whether the window is actually visible.
  pub fn is_visible(&self) -> crate::Result<bool> {
    self.inner.is_visible()
  }

  /// Whether the window is minimized.
  pub fn is_minimized(&self) -> crate::Result<bool> {
    self.inner.is_minimized()
  }

  /// Resize the window to the specified size.
  pub fn resize(&self, width: f64, height: f64) -> crate::Result<()> {
    self.inner.resize(width, height)
  }
}

impl PartialEq for NativeWindow {
  fn eq(&self, other: &Self) -> bool {
    self.inner.handle == other.inner.handle
  }
}

impl Eq for NativeWindow {}
