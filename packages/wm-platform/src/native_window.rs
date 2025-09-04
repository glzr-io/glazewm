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
  AfterWindow(WindowId),
  Top,
  TopMost,
}

#[derive(Clone, Debug)]
pub struct NativeWindow {
  pub(crate) inner: platform_impl::NativeWindow,
}

impl NativeWindow {
  #[must_use]
  pub fn id(&self) -> WindowId {
    self.inner.id()
  }

  /// Gets the window's title. If the window is invalid, returns an empty
  /// string.
  pub fn title(&self) -> crate::Result<String> {
    // self.inner.title()
    Ok("test".to_string())
  }

  // TODO: Remove this (should only be on `NativeWindowWindowsExt`).
  pub fn class_name(&self) -> crate::Result<String> {
    Ok("test".to_string())
  }

  // TODO: Remove this (should only be on `NativeWindowWindowsExt`).
  pub fn process_name(&self) -> crate::Result<String> {
    Ok("test".to_string())
  }

  pub fn frame(&self) -> crate::Result<Rect> {
    self.inner.frame()
  }

  pub fn position(&self) -> crate::Result<(f64, f64)> {
    self.inner.position()
  }

  pub fn size(&self) -> crate::Result<(f64, f64)> {
    self.inner.size()
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

  pub fn reposition(&self, x: f64, y: f64) -> crate::Result<()> {
    self.inner.reposition(x, y)
  }

  pub fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    self.inner.set_frame(rect)
  }

  pub fn minimize(&self) -> crate::Result<()> {
    self.inner.minimize()
  }

  pub fn is_fullscreen(&self, monitor_rect: &Rect) -> crate::Result<bool> {
    // TODO: Implement this.
    Ok(false)
  }

  pub fn is_resizable(&self) -> crate::Result<bool> {
    // TODO: Implement this.
    Ok(true)
  }
}

impl PartialEq for NativeWindow {
  fn eq(&self, other: &Self) -> bool {
    self.inner.handle == other.inner.handle
  }
}

impl Eq for NativeWindow {}
