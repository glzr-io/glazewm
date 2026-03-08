#[cfg(target_os = "macos")]
use objc2_application_services::AXUIElement;
#[cfg(target_os = "macos")]
use objc2_core_foundation::{CFBoolean, CFRetained, CFString};
#[cfg(target_os = "windows")]
use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::{
    SET_WINDOW_POS_FLAGS, WINDOW_EX_STYLE, WINDOW_STYLE,
  },
};

use crate::{platform_impl, Rect};
#[cfg(target_os = "macos")]
use crate::{platform_impl::AXUIElementExt, ThreadBound};
#[cfg(target_os = "windows")]
use crate::{Color, CornerStyle, Delta, OpacityValue, RectDelta};

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
pub enum WindowZOrder {
  Normal,
  AfterWindow(WindowId),
  Top,
  TopMost,
}

/// macOS-specific extension trait for [`NativeWindow`].
#[cfg(target_os = "macos")]
pub trait NativeWindowExtMacOs {
  /// Gets the `AXUIElement` instance for this window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn ax_ui_element(&self) -> &ThreadBound<CFRetained<AXUIElement>>;

  /// Gets the bundle ID of the application that owns the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn bundle_id(&self) -> Option<String>;

  /// Gets the role of the window (e.g. `AXWindow`).
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn role(&self) -> crate::Result<String>;

  /// Gets the sub-role of the window (e.g. `AXStandardWindow`).
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn subrole(&self) -> crate::Result<String>;

  /// Whether the window is modal.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn is_modal(&self) -> crate::Result<bool>;

  /// Whether the window is the main window for its application.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn is_main(&self) -> crate::Result<bool>;
}

#[cfg(target_os = "macos")]
impl NativeWindowExtMacOs for NativeWindow {
  fn ax_ui_element(&self) -> &ThreadBound<CFRetained<AXUIElement>> {
    &self.inner.element
  }

  fn bundle_id(&self) -> Option<String> {
    self.inner.application.bundle_id()
  }

  fn role(&self) -> crate::Result<String> {
    self.inner.element.with(|el| {
      el.get_attribute::<CFString>("AXRole")
        .map(|cf_string| cf_string.to_string())
    })?
  }

  fn subrole(&self) -> crate::Result<String> {
    self.inner.element.with(|el| {
      el.get_attribute::<CFString>("AXSubrole")
        .map(|cf_string| cf_string.to_string())
    })?
  }

  fn is_modal(&self) -> crate::Result<bool> {
    self.inner.element.with(|el| {
      el.get_attribute::<CFBoolean>("AXModal")
        .map(|cf_bool| cf_bool.value())
    })?
  }

  fn is_main(&self) -> crate::Result<bool> {
    self.inner.element.with(|el| {
      el.get_attribute::<CFBoolean>("AXMain")
        .map(|cf_bool| cf_bool.value())
    })?
  }
}

/// Windows-specific extensions for [`NativeWindow`].
#[cfg(target_os = "windows")]
pub trait NativeWindowWindowsExt {
  /// Creates a [`NativeWindow`] from a window handle.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn from_handle(handle: isize) -> NativeWindow;

  /// Gets the window handle.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn hwnd(&self) -> HWND;

  /// Gets the class name of the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn class_name(&self) -> crate::Result<String>;

  /// Gets the window's frame, including the window's shadow borders.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn frame_with_shadows(&self) -> crate::Result<Rect>;

  /// Gets the delta between the window's frame and the window's border.
  /// This represents the size of a window's shadow borders.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn shadow_borders(&self) -> crate::Result<RectDelta>;

  /// Whether the window has an owner window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn has_owner_window(&self) -> bool;

  /// Whether the window has the given window style flag(s) set.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn has_window_style(&self, style: WINDOW_STYLE) -> bool;

  /// Whether the window has the given extended window style flag(s) set.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn has_window_style_ex(&self, style: WINDOW_EX_STYLE) -> bool;

  /// Thin wrapper around [`SetWindowPos`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos).
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_window_pos(
    &self,
    z_order: &WindowZOrder,
    rect: &Rect,
    flags: SET_WINDOW_POS_FLAGS,
  ) -> crate::Result<()>;

  /// Shows the window asynchronously.
  ///
  /// NOTE: Cloaked windows do not get shown until uncloaked.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn show(&self) -> crate::Result<()>;

  /// Hides the window asynchronously.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn hide(&self) -> crate::Result<()>;

  /// Restores the window (unminimizes and unmaximizes).
  ///
  /// If `outer_frame` is provided, the window will be restored to the
  /// specified position. This avoids flickering compared to restoring
  /// and then repositioning the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn restore(&self, outer_frame: Option<&Rect>) -> crate::Result<()>;

  /// Cloaks or uncloaks the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_cloaked(&self, cloaked: bool) -> crate::Result<()>;

  /// Marks the window as fullscreen.
  ///
  /// Causes the native Windows taskbar to be moved to the bottom of the
  /// z-order when this window is active.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn mark_fullscreen(&self, fullscreen: bool) -> crate::Result<()>;

  /// Adds or removes the window from the native taskbar.
  ///
  /// Cloaked windows are normally always shown in the taskbar, but can be
  /// manually toggled. Hidden windows (`SW_HIDE`) can never be shown in
  /// the taskbar.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_taskbar_visibility(&self, visible: bool) -> crate::Result<()>;

  /// Adds the given extended window style flag(s) to the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn add_window_style_ex(&self, style: WINDOW_EX_STYLE);

  /// Sets the window's z-order.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_z_order(&self, zorder: &WindowZOrder) -> crate::Result<()>;

  /// Sets the visibility of the window's title bar.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_title_bar_visibility(&self, visible: bool) -> crate::Result<()>;

  /// Sets the color of the window's border.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_border_color(&self, color: Option<&Color>) -> crate::Result<()>;

  /// Sets the corner style of the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_corner_style(
    &self,
    corner_style: &CornerStyle,
  ) -> crate::Result<()>;

  /// Sets the transparency of the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn set_transparency(
    &self,
    opacity_value: &OpacityValue,
  ) -> crate::Result<()>;

  /// Adjusts the window's transparency by a relative delta.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn adjust_transparency(
    &self,
    opacity_delta: &Delta<OpacityValue>,
  ) -> crate::Result<()>;
}

#[cfg(target_os = "windows")]
impl NativeWindowWindowsExt for NativeWindow {
  fn from_handle(handle: isize) -> Self {
    platform_impl::NativeWindow::new(handle).into()
  }

  fn hwnd(&self) -> HWND {
    self.inner.hwnd()
  }

  fn class_name(&self) -> crate::Result<String> {
    self.inner.class_name()
  }

  fn frame_with_shadows(&self) -> crate::Result<Rect> {
    self.inner.frame_with_shadows()
  }

  fn shadow_borders(&self) -> crate::Result<RectDelta> {
    self.inner.shadow_borders()
  }

  fn has_owner_window(&self) -> bool {
    self.inner.has_owner_window()
  }

  fn has_window_style(&self, style: WINDOW_STYLE) -> bool {
    self.inner.has_window_style(style)
  }

  fn has_window_style_ex(&self, style: WINDOW_EX_STYLE) -> bool {
    self.inner.has_window_style_ex(style)
  }

  fn set_window_pos(
    &self,
    z_order: &WindowZOrder,
    rect: &Rect,
    flags: SET_WINDOW_POS_FLAGS,
  ) -> crate::Result<()> {
    self.inner.set_window_pos(z_order, rect, flags)
  }

  fn show(&self) -> crate::Result<()> {
    self.inner.show()
  }

  fn hide(&self) -> crate::Result<()> {
    self.inner.hide()
  }

  fn restore(&self, outer_frame: Option<&Rect>) -> crate::Result<()> {
    self.inner.restore(outer_frame)
  }

  fn set_cloaked(&self, cloaked: bool) -> crate::Result<()> {
    self.inner.set_cloaked(cloaked)
  }

  fn mark_fullscreen(&self, fullscreen: bool) -> crate::Result<()> {
    self.inner.mark_fullscreen(fullscreen)
  }

  fn set_taskbar_visibility(&self, visible: bool) -> crate::Result<()> {
    self.inner.set_taskbar_visibility(visible)
  }

  fn add_window_style_ex(&self, style: WINDOW_EX_STYLE) {
    self.inner.add_window_style_ex(style);
  }

  fn set_z_order(&self, z_order: &WindowZOrder) -> crate::Result<()> {
    self.inner.set_z_order(z_order)
  }

  fn set_title_bar_visibility(&self, visible: bool) -> crate::Result<()> {
    self.inner.set_title_bar_visibility(visible)
  }

  fn set_border_color(&self, color: Option<&Color>) -> crate::Result<()> {
    self.inner.set_border_color(color)
  }

  fn set_corner_style(
    &self,
    corner_style: &CornerStyle,
  ) -> crate::Result<()> {
    self.inner.set_corner_style(corner_style)
  }

  fn set_transparency(
    &self,
    opacity_value: &OpacityValue,
  ) -> crate::Result<()> {
    self.inner.set_transparency(opacity_value)
  }

  fn adjust_transparency(
    &self,
    opacity_delta: &Delta<OpacityValue>,
  ) -> crate::Result<()> {
    self.inner.adjust_transparency(opacity_delta)
  }
}

#[derive(Clone, Debug)]
pub struct NativeWindow {
  pub(crate) inner: platform_impl::NativeWindow,
}

impl NativeWindow {
  /// Gets the unique identifier for this window.
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
  /// Returns [`Error::WindowNotFound`] if the window is invalid.
  pub fn title(&self) -> crate::Result<String> {
    self.inner.title()
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

  /// Gets the window's position as (x, y) coordinates.
  pub fn position(&self) -> crate::Result<(f64, f64)> {
    self.inner.position()
  }

  /// Gets the window's size as (width, height).
  pub fn size(&self) -> crate::Result<(f64, f64)> {
    self.inner.size()
  }

  /// Whether the window is still valid.
  ///
  /// Returns `true` if the underlying window is still alive.
  #[must_use]
  pub fn is_valid(&self) -> bool {
    self.inner.is_valid()
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

  /// Whether the window can be resized.
  pub fn is_resizable(&self) -> crate::Result<bool> {
    self.inner.is_resizable()
  }

  /// Whether the window is the OS's desktop window.
  pub fn is_desktop_window(&self) -> crate::Result<bool> {
    self.inner.is_desktop_window()
  }

  /// Repositions and resizes the window to the specified rectangle.
  ///
  /// # Platform-specific
  ///
  /// - **Windows**: Automatically adjusts the `rect` prior to calling [`SetWindowPos`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos)
  ///   to include the window's shadow borders. To set the window's
  ///   position directly, use [`NativeWindowWindowsExt::set_window_pos`].
  pub fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    self.inner.set_frame(rect)
  }

  /// Resizes the window to the specified size.
  pub fn resize(&self, width: i32, height: i32) -> crate::Result<()> {
    self.inner.resize(width, height)
  }

  /// Repositions the window to the specified position.
  pub fn reposition(&self, x: i32, y: i32) -> crate::Result<()> {
    self.inner.reposition(x, y)
  }

  pub fn minimize(&self) -> crate::Result<()> {
    self.inner.minimize()
  }

  pub fn maximize(&self) -> crate::Result<()> {
    self.inner.maximize()
  }

  /// Sets focus to the window and raises it to the top of the z-order.
  pub fn focus(&self) -> crate::Result<()> {
    self.inner.focus()
  }

  /// Closes the window.
  ///
  /// # Platform-specific
  ///
  /// - **Windows**: This sends a `WM_CLOSE` message to the window.
  /// - **macOS**: This simulates pressing the close button on the window's
  ///   title bar.
  pub fn close(&self) -> crate::Result<()> {
    self.inner.close()
  }
}

impl PartialEq for NativeWindow {
  fn eq(&self, other: &Self) -> bool {
    self.inner.id() == other.inner.id()
  }
}

impl Eq for NativeWindow {}
