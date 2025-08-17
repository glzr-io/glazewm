#[cfg(target_os = "macos")]
use objc2_app_kit::NSScreen;
#[cfg(target_os = "macos")]
use objc2_core_graphics::CGDirectDisplayID;

#[cfg(target_os = "macos")]
use crate::{
  display::{Display, DisplayDevice},
  Result,
};

/// Placeholder for MainThreadRef type - would need proper implementation.
#[cfg(target_os = "macos")]
pub struct MainThreadRef<T>(pub T);

/// Placeholder for CFRetained type - would need proper implementation.
#[cfg(target_os = "macos")]
pub struct CFRetained<T>(pub T);

/// Reference to a Metal device on macOS.
#[cfg(target_os = "macos")]
#[derive(Clone, Debug)]
pub struct MetalDeviceRef {
  /// Internal Metal device reference.
  pub(crate) inner: u64, // Placeholder
}

/// macOS-specific extensions for `Display`.
///
/// This trait provides access to platform-specific functionality
/// that is only available on macOS.
#[cfg(target_os = "macos")]
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
#[cfg(target_os = "macos")]
pub trait DisplayDeviceExtMacos {
  /// Gets the display unit number.
  ///
  /// The unit number is a macOS-specific identifier for the
  /// display device.
  ///
  /// # Platform-specific
  /// This method is only available on macOS.
  fn unit_number(&self) -> Result<Option<u32>>;

  /// Gets the GPU registry ID.
  ///
  /// The registry ID uniquely identifies the GPU in the
  /// macOS IOKit registry. Only available for physical devices.
  ///
  /// # Platform-specific
  /// This method is only available on macOS.
  fn registry_id(&self) -> Result<Option<u64>>;

  /// Gets a reference to the Metal device.
  ///
  /// Provides access to the Metal device associated with this
  /// display device. Only available for physical devices.
  ///
  /// # Platform-specific
  /// This method is only available on macOS.
  fn metal_device(&self) -> Result<Option<MetalDeviceRef>>;
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

#[cfg(target_os = "macos")]
impl DisplayDeviceExtMacos for DisplayDevice {
  fn unit_number(&self) -> Result<Option<u32>> {
    self.inner.unit_number()
  }

  fn registry_id(&self) -> Result<Option<u64>> {
    self.inner.registry_id()
  }

  fn metal_device(&self) -> Result<Option<MetalDeviceRef>> {
    self.inner.metal_device()
  }
}
