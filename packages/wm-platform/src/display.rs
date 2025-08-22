use wm_common::Rect;

use crate::{platform_impl, Result};

/// Unique identifier for a display.
///
/// Can be obtained with `display.id()`.
///
/// # Platform-specific
///
/// - **Windows**: `isize` (`HMONITOR`)
/// - **macOS**: `u32` (`CGDirectDisplayID`)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayId(
  #[cfg(target_os = "windows")] pub(crate) isize,
  #[cfg(target_os = "macos")] pub(crate) u32,
);

/// Unique identifier for a display device.
///
/// Can be obtained with `display_device.id()`.
///
/// # Platform-specific
///
/// - **Windows**: Hardware ID string
/// - **macOS**: `u32` (CGUUID as u32)
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayDeviceId(
  #[cfg(target_os = "windows")] pub(crate) String,
  #[cfg(target_os = "macos")] pub(crate) u32,
);

/// Represents a logical display space where windows can be placed.
///
/// # Platform-specific
///
/// - **Windows**: This corresponds to a "display monitor", each with a
///   monitor handle (`HMONITOR`).
/// - **macOS**: This corresponds to an `NSScreen`.
/// TODO: Add `PartialEq` and `Eq`.
#[derive(Clone, Debug)]
pub struct Display {
  pub(crate) inner: platform_impl::Display,
}

impl Display {
  /// Gets the unique identifier for this display.
  pub fn id(&self) -> DisplayId {
    self.inner.id()
  }

  /// Gets the display name.
  pub fn name(&self) -> Result<String> {
    self.inner.name()
  }

  /// Gets the full bounds rectangle of the display.
  pub fn bounds(&self) -> Result<Rect> {
    self.inner.bounds()
  }

  /// Gets the working area rectangle (excluding system UI).
  pub fn working_area(&self) -> Result<Rect> {
    self.inner.working_area()
  }

  /// Gets the scale factor for the display.
  pub fn scale_factor(&self) -> Result<f32> {
    self.inner.scale_factor()
  }

  /// Gets the DPI for the display.
  pub fn dpi(&self) -> Result<u32> {
    self.inner.dpi()
  }

  /// Returns whether this is the primary display.
  pub fn is_primary(&self) -> Result<bool> {
    self.inner.is_primary()
  }

  /// Gets the display devices for this display.
  ///
  /// A single display can be associated with multiple display devices. For
  /// example, when mirroring a display or combining multiple displays
  /// (e.g. using NVIDIA Surround).
  pub fn devices(&self) -> Result<Vec<DisplayDevice>> {
    self.inner.devices()
  }

  /// Gets the main device (first non-mirroring device) for this display.
  pub fn main_device(&self) -> Result<Option<DisplayDevice>> {
    self.inner.main_device()
  }
}

/// Connection state of a display device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionState {
  /// Device is connected and part of the desktop coordinate space.
  Active,

  /// Device is connected but inactive (e.g. on standby or in sleep mode).
  Inactive,

  /// Device is disconnected.
  Disconnected,
}

/// Mirroring state of a display device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MirroringState {
  /// This device is the source being mirrored.
  Source,

  /// This device is mirroring another (target).
  Target,
}

/// Display connection type for physical devices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputTechnology {
  /// Built-in display (laptop screen).
  Internal,
  /// VGA connection.
  VGA,
  /// DVI connection.
  DVI,
  /// HDMI connection.
  HDMI,
  /// DisplayPort connection.
  DisplayPort,
  /// Thunderbolt connection.
  Thunderbolt,
  /// USB connection.
  USB,
  /// Wireless connection.
  Wireless,
  /// Unknown connection type.
  Unknown,
}

/// Represents a display adapter/device (physical or virtual).
///
/// This is typically a physical display device, such as a monitor or
/// built-in laptop screen.
///
/// TODO: Add `PartialEq` and `Eq`.
#[derive(Clone, Debug)]
pub struct DisplayDevice {
  pub(crate) inner: platform_impl::DisplayDevice,
}

impl DisplayDevice {
  /// Gets the unique identifier for this display device.
  pub fn id(&self) -> DisplayDeviceId {
    self.inner.id()
  }

  /// Gets the rotation of the device in degrees.
  pub fn rotation(&self) -> Result<f32> {
    self.inner.rotation()
  }

  /// Gets whether this is a built-in display.
  ///
  /// Returns `true` for embedded displays (like laptop screens).
  pub fn is_builtin(&self) -> Result<bool> {
    self.inner.is_builtin()
  }

  /// Gets the connection state of the device.
  pub fn connection_state(&self) -> Result<ConnectionState> {
    self.inner.connection_state()
  }

  /// Gets the mirroring state of the device.
  pub fn mirroring_state(&self) -> Result<Option<MirroringState>> {
    self.inner.mirroring_state()
  }

  /// Gets the refresh rate of the device in Hz.
  pub fn refresh_rate(&self) -> Result<f32> {
    self.inner.refresh_rate()
  }
}

#[cfg(test)]
mod tests {}
