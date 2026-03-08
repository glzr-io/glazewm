#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2_app_kit::NSScreen;
#[cfg(target_os = "macos")]
use objc2_core_graphics::CGDirectDisplayID;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::HMONITOR;

#[cfg(target_os = "macos")]
use crate::ThreadBound;
use crate::{platform_impl, Rect};

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
  #[cfg(target_os = "windows")] pub isize,
  #[cfg(target_os = "macos")] pub u32,
);

/// Unique identifier for a display device.
///
/// Can be obtained with `display_device.id()`.
///
/// # Platform-specific
///
/// - **Windows**: Hardware ID string with fallback to adapter name.
/// - **macOS**: UUID string from `CGDisplayCreateUUIDFromDisplayID`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayDeviceId(pub String);

/// macOS-specific extension trait for [`Display`].
#[cfg(target_os = "macos")]
pub trait DisplayExtMacOs {
  /// Gets the Core Graphics display ID.
  fn cg_display_id(&self) -> CGDirectDisplayID;

  /// Gets the `NSScreen` instance for this display.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn ns_screen(&self) -> &ThreadBound<Retained<NSScreen>>;
}

#[cfg(target_os = "macos")]
impl DisplayExtMacOs for Display {
  fn cg_display_id(&self) -> CGDirectDisplayID {
    self.inner.cg_display_id()
  }

  fn ns_screen(&self) -> &ThreadBound<Retained<NSScreen>> {
    self.inner.ns_screen()
  }
}

/// Windows-specific extensions for [`Display`].
#[cfg(target_os = "windows")]
pub trait DisplayExtWindows {
  /// Gets the monitor handle.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn hmonitor(&self) -> HMONITOR;
}

#[cfg(target_os = "windows")]
impl DisplayExtWindows for Display {
  fn hmonitor(&self) -> HMONITOR {
    self.inner.hmonitor()
  }
}

/// Represents a logical display space where windows can be placed.
///
/// # Platform-specific
///
/// - **Windows**: This corresponds to a Win32 "display monitor", each with
///   a monitor handle (`HMONITOR`).
/// - **macOS**: This corresponds to an `NSScreen`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Display {
  pub(crate) inner: platform_impl::Display,
}

impl Display {
  /// Gets the unique identifier for this display.
  #[must_use]
  pub fn id(&self) -> DisplayId {
    self.inner.id()
  }

  /// Gets the display name.
  pub fn name(&self) -> crate::Result<String> {
    self.inner.name()
  }

  /// Gets the full bounds rectangle of the display.
  pub fn bounds(&self) -> crate::Result<Rect> {
    self.inner.bounds()
  }

  /// Gets the working area rectangle (excluding system UI).
  pub fn working_area(&self) -> crate::Result<Rect> {
    self.inner.working_area()
  }

  /// Gets the scale factor for the display.
  pub fn scale_factor(&self) -> crate::Result<f32> {
    self.inner.scale_factor()
  }

  /// Gets the DPI for the display.
  pub fn dpi(&self) -> crate::Result<u32> {
    self.inner.dpi()
  }

  /// Returns whether this is the primary display.
  pub fn is_primary(&self) -> crate::Result<bool> {
    self.inner.is_primary()
  }

  /// Gets the display devices for this display.
  ///
  /// A single display can be associated with multiple display devices. For
  /// example, when mirroring a display or combining multiple displays
  /// (e.g. using NVIDIA Surround).
  pub fn devices(&self) -> crate::Result<Vec<DisplayDevice>> {
    self.inner.devices()
  }

  /// Gets the main device (first non-mirroring device) for this display.
  pub fn main_device(&self) -> crate::Result<DisplayDevice> {
    self.inner.main_device()
  }
}

/// Connection state of a display device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionState {
  /// Device is connected and part of the desktop coordinate space.
  Active,

  /// Device is connected but inactive (i.e. on standby or in sleep mode).
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

/// macOS-specific extension trait for [`DisplayDevice`].
#[cfg(target_os = "macos")]
pub trait DisplayDeviceExtMacOs {
  /// Gets the Core Graphics display ID.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn cg_display_id(&self) -> CGDirectDisplayID;
}

#[cfg(target_os = "macos")]
impl DisplayDeviceExtMacOs for DisplayDevice {
  fn cg_display_id(&self) -> CGDirectDisplayID {
    self.inner.cg_display_id()
  }
}

/// Windows-specific extensions for [`DisplayDevice`].
#[cfg(target_os = "windows")]
pub trait DisplayDeviceExtWindows {
  /// Gets the device path.
  ///
  /// This can be an empty string for virtual display devices.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn device_path(&self) -> Option<String>;

  /// Gets the hardware ID from the device path.
  ///
  /// # Example usage
  ///
  /// ```rust,no_run
  /// device.device_path(); // "\\?\DISPLAY#DEL40A3#5&1234abcd&0&UID256#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}"
  /// device.hardware_id(); // Some("DEL40A3")
  /// ```
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn hardware_id(&self) -> Option<String>;

  /// Gets the output technology.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  fn output_technology(&self) -> crate::Result<Option<OutputTechnology>>;
}

#[cfg(target_os = "windows")]
impl DisplayDeviceExtWindows for DisplayDevice {
  fn device_path(&self) -> Option<String> {
    self.inner.device_path.clone()
  }

  fn hardware_id(&self) -> Option<String> {
    self.inner.hardware_id()
  }

  fn output_technology(&self) -> crate::Result<Option<OutputTechnology>> {
    self.inner.output_technology()
  }
}

/// Represents a display device (physical or virtual).
///
/// This is typically a physical display device, such as a monitor or
/// built-in laptop screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayDevice {
  pub(crate) inner: platform_impl::DisplayDevice,
}

impl DisplayDevice {
  /// Gets the unique identifier for this display device.
  #[must_use]
  pub fn id(&self) -> DisplayDeviceId {
    self.inner.id()
  }

  /// Gets the rotation of the device in degrees.
  pub fn rotation(&self) -> crate::Result<f32> {
    self.inner.rotation()
  }

  /// Gets the refresh rate of the device in Hz.
  pub fn refresh_rate(&self) -> crate::Result<f32> {
    self.inner.refresh_rate()
  }

  /// Gets whether this is a built-in display.
  ///
  /// Returns `true` for embedded displays (like laptop screens).
  pub fn is_builtin(&self) -> crate::Result<bool> {
    self.inner.is_builtin()
  }

  /// Gets the connection state of the device.
  pub fn connection_state(&self) -> crate::Result<ConnectionState> {
    self.inner.connection_state()
  }

  /// Gets the mirroring state of the device.
  pub fn mirroring_state(&self) -> crate::Result<Option<MirroringState>> {
    self.inner.mirroring_state()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::EventLoop;

  #[test]
  fn test_nearest_display() {
    let (event_loop, dispatcher) = EventLoop::new().unwrap();

    let thread = std::thread::spawn(move || {
      let display = platform_impl::nearest_display(
        // Assumes that there is at least one window currently visible.
        &dispatcher.visible_windows().unwrap()[0],
        &dispatcher,
      );
      dispatcher.stop_event_loop().unwrap();
      display
    });

    event_loop.run().unwrap();
    let display = thread.join().unwrap();
    assert!(display.is_ok());
  }
}
