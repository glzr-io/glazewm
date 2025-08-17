#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::HMONITOR;

#[cfg(target_os = "windows")]
use crate::{
  display::{Display, DisplayDevice},
  Result,
};

/// Windows-specific display settings.
#[cfg(target_os = "windows")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowsDisplaySettings {
  /// Display mode width.
  pub width: u32,
  /// Display mode height.
  pub height: u32,
  /// Color depth in bits per pixel.
  pub bits_per_pixel: u32,
  /// Refresh rate in Hz.
  pub refresh_rate: u32,
  /// Display orientation.
  pub orientation: DisplayOrientation,
}

/// Display orientation on Windows.
#[cfg(target_os = "windows")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayOrientation {
  /// Default orientation.
  Default,
  /// Rotated 90 degrees.
  Rotate90,
  /// Rotated 180 degrees.
  Rotate180,
  /// Rotated 270 degrees.
  Rotate270,
}

/// Windows-specific extensions for `Display`.
///
/// This trait provides access to platform-specific functionality
/// that is only available on Windows.
#[cfg(target_os = "windows")]
pub trait DisplayExtWindows {
  /// Gets the Windows monitor handle.
  fn hmonitor(&self) -> HMONITOR;

  /// Gets Windows-specific display settings.
  ///
  /// # Platform-specific
  /// This method is only available on Windows.
  fn display_settings(&self) -> Result<WindowsDisplaySettings>;
}

/// Windows-specific extensions for `DisplayDevice`.
///
/// This trait provides access to platform-specific functionality
/// that is only available on Windows.
#[cfg(target_os = "windows")]
pub trait DisplayDeviceExtWindows {
  /// Gets the device path for the display adapter.
  ///
  /// This is the DOS device path used by the display driver.
  ///
  /// # Platform-specific
  /// This method is only available on Windows.
  fn adapter_device_path(&self) -> Result<Option<String>>;

  /// Gets the device instance ID.
  ///
  /// The device instance ID uniquely identifies the device
  /// in the Windows device tree.
  ///
  /// # Platform-specific
  /// This method is only available on Windows.
  fn device_instance_id(&self) -> Result<Option<String>>;

  /// Gets the hardware vendor and device IDs.
  ///
  /// Returns a tuple of (vendor_id, device_id) if available.
  /// These are the PCI vendor and device identifiers.
  ///
  /// # Platform-specific
  /// This method is only available on Windows.
  fn hardware_ids(&self) -> Result<Option<(u16, u16)>>;

  /// Gets the registry key for device properties.
  ///
  /// This is the registry path where additional device properties
  /// can be found.
  ///
  /// # Platform-specific
  /// This method is only available on Windows.
  fn registry_key(&self) -> Result<Option<String>>;

  /// Checks if the device is built-in using Windows output type.
  ///
  /// Uses Windows-specific APIs to determine if this is an
  /// internal display adapter.
  ///
  /// # Platform-specific
  /// This method is only available on Windows.
  fn is_builtin_windows(&self) -> Result<bool>;
}

#[cfg(target_os = "windows")]
impl DisplayExtWindows for Display {
  fn hmonitor(&self) -> HMONITOR {
    self.inner.hmonitor()
  }

  fn display_settings(&self) -> Result<WindowsDisplaySettings> {
    self.inner.display_settings()
  }
}

#[cfg(target_os = "windows")]
impl DisplayDeviceExtWindows for DisplayDevice {
  fn adapter_device_path(&self) -> Result<Option<String>> {
    self.inner.adapter_device_path()
  }

  fn device_instance_id(&self) -> Result<Option<String>> {
    self.inner.device_instance_id()
  }

  fn hardware_ids(&self) -> Result<Option<(u16, u16)>> {
    self.inner.hardware_ids()
  }

  fn registry_key(&self) -> Result<Option<String>> {
    self.inner.registry_key()
  }

  fn is_builtin_windows(&self) -> Result<bool> {
    self.inner.is_builtin_windows()
  }
}
