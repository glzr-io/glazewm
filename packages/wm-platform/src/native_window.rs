#[cfg(target_os = "macos")]
use objc2_application_services::AXUIElement;
#[cfg(target_os = "macos")]
use objc2_core_foundation::CFRetained;

use crate::{platform_impl, Rect};

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
  #[cfg(target_os = "windows")] pub isize,
  #[cfg(target_os = "macos")] pub u32,
);

impl WindowId {
  #[cfg(target_os = "macos")]
  pub(crate) fn from_window_element(el: &CFRetained<AXUIElement>) -> Self {
    let mut window_id = 0;

    unsafe {
      platform_impl::ffi::_AXUIElementGetWindow(
        CFRetained::as_ptr(el),
        &raw mut window_id,
      )
    };

    Self(window_id)
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

  /// Gets the window's title.
  ///
  /// Note that empty strings are valid window titles.
  ///
  /// # Errors
  ///
  /// Returns `crate::Error::WindowNotFound` if the window is invalid.
  pub fn title(&self) -> crate::Result<String> {
    self.inner.title()
  }

  // TODO: Remove this (should only be on `NativeWindowWindowsExt`).
  pub fn class_name(&self) -> crate::Result<String> {
    Ok("test".to_string())
  }

  pub fn process_name(&self) -> crate::Result<String> {
    self.inner.process_name()
  }

  /// Gets a rectangle of the window's size and position.
  ///
  /// # Platform-specific
  ///
  /// - **Windows**: Includes the window's shadow borders.
  /// - **macOS**: If the window was previously resized to a value outside
  ///   of the window's allowed min/max width & height (e.g. via calling
  ///   `set_frame`), this can return those invalid values and might not
  ///   reflect the actual window size.
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

  /// Whether the window is maximized.
  pub fn is_maximized(&self) -> crate::Result<bool> {
    self.inner.is_maximized()
  }

  /// Resizes the window to the specified size.
  pub fn resize(&self, width: i32, height: i32) -> crate::Result<()> {
    self.inner.resize(width, height)
  }

  /// Repositions the window to the specified position.
  pub fn reposition(&self, x: i32, y: i32) -> crate::Result<()> {
    self.inner.reposition(x, y)
  }

  /// Repositions and resizes the window to the specified rectangle.
  ///
  /// # Platform-specific
  ///
  /// - **Windows**: Automatically adjusts the `rect` prior to calling [`SetWindowPos`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos)
  ///   to include the window's shadow borders. To set the window's frame
  ///   without the shadow borders, use
  ///   [`Self::set_frame_without_shadow_borders`].
  pub fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    self.inner.set_frame(rect)
  }

  pub fn minimize(&self) -> crate::Result<()> {
    self.inner.minimize()
  }

  pub fn maximize(&self) -> crate::Result<()> {
    self.inner.maximize()
  }

  /// Closes the window.
  ///
  /// # Platform-specific
  ///
  /// - **Windows**: This sends a `WM_CLOSE` message to the window.
  /// - **macOS**: This simulates pressing the close button on the window's
  ///   title bar.
  ///
  /// # Errors
  ///
  /// Returns `crate::Error::WindowNotFound` if the window is invalid or
  /// cannot be closed.
  pub fn close(&self) -> crate::Result<()> {
    self.inner.close()
  }

  pub fn is_resizable(&self) -> crate::Result<bool> {
    // TODO: Implement this.
    Ok(true)
  }

  pub fn is_desktop_window(&self) -> crate::Result<bool> {
    self.inner.is_desktop_window()
  }

  /// Sets focus to the window and raises it to the top of the z-order.
  pub fn focus(&self) -> crate::Result<()> {
    self.inner.focus()
  }
}

impl PartialEq for NativeWindow {
  fn eq(&self, other: &Self) -> bool {
    self.inner.id() == other.inner.id()
  }
}

impl Eq for NativeWindow {}
