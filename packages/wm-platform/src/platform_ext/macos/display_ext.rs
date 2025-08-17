use objc2_app_kit::NSScreen;
use objc2_core_foundation::CFRetained;
use objc2_core_graphics::CGDirectDisplayID;

use crate::{
  display::{Display, DisplayDevice},
  platform_impl::MainThreadRef,
  Result,
};

/// macOS-specific extensions for `Display`.
///
/// This trait provides access to platform-specific functionality
/// that is only available on macOS.
pub trait DisplayExtMacos {
  /// Gets the Core Graphics display ID.
  fn cg_display_id(&self) -> CGDirectDisplayID;

  /// Gets the NSScreen instance for this display.
  ///
  /// NSScreen is always available for active displays. This method
  /// provides thread-safe access to the NSScreen.
  ///
  /// # Platform-specific
  /// This method is only available on macOS.
  fn ns_screen(&self) -> Result<MainThreadRef<CFRetained<NSScreen>>>;

  /// Checks if this is a built-in display.
  ///
  /// Returns true for internal displays (like laptop screens).
  ///
  /// # Platform-specific
  /// This method is only available on macOS.
  fn is_builtin(&self) -> Result<bool>;
}

/// macOS-specific extensions for `DisplayDevice`.
///
/// This trait provides access to platform-specific functionality
/// that is only available on macOS.
pub trait DisplayDeviceExtMacos {
  /// Gets the Core Graphics display ID.
  fn cg_display_id(&self) -> CGDirectDisplayID;
}

#[cfg(target_os = "macos")]
impl DisplayExtMacos for Display {
  fn cg_display_id(&self) -> CGDirectDisplayID {
    self.inner.cg_display_id()
  }

  fn ns_screen(&self) -> Result<MainThreadRef<CFRetained<NSScreen>>> {
    self.inner.ns_screen()
  }

  fn is_builtin(&self) -> Result<bool> {
    self.inner.is_builtin()
  }
}

impl DisplayDeviceExtMacos for DisplayDevice {
  fn cg_display_id(&self) -> CGDirectDisplayID {
    self.inner.cg_display_id()
  }
}
