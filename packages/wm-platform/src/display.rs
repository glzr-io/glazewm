use wm_common::{Point, Rect};

use crate::{platform_impl, Result};

/// Unique identifier for a display.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DisplayId(String);

impl DisplayId {
  /// Creates a new display ID.
  #[must_use]
  pub fn new(id: impl Into<String>) -> Self {
    Self(id.into())
  }

  /// Gets the string representation of the display ID.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

/// Unique identifier for a display device.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DisplayDeviceId(String);

impl DisplayDeviceId {
  /// Creates a new display device ID.
  #[must_use]
  pub fn new(id: impl Into<String>) -> Self {
    Self(id.into())
  }

  /// Gets the string representation of the display device ID.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

/// Represents an active display output.
///
/// Displays are always active - inactive displays are not enumerated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Display {
  pub(crate) inner: platform_impl::Display,
}

impl Display {
  /// Creates a new `Display` from platform-specific data.
  #[must_use]
  pub(crate) fn from_platform_impl(inner: platform_impl::Display) -> Self {
    Self { inner }
  }

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

  /// Gets the display resolution in pixels.
  pub fn resolution(&self) -> Result<(u32, u32)> {
    self.inner.resolution()
  }

  /// Gets the current refresh rate in Hz.
  pub fn refresh_rate(&self) -> Result<f32> {
    self.inner.refresh_rate()
  }

  /// Gets the scale factor for the display.
  pub fn scale_factor(&self) -> Result<f32> {
    self.inner.scale_factor()
  }

  /// Gets the DPI for the display.
  pub fn dpi(&self) -> Result<u32> {
    self.inner.dpi()
  }

  /// Gets the bit depth of the display.
  pub fn bit_depth(&self) -> Result<u32> {
    self.inner.bit_depth()
  }

  /// Returns whether this is the primary display.
  pub fn is_primary(&self) -> Result<bool> {
    self.inner.is_primary()
  }

  /// Returns whether this display supports HDR.
  pub fn is_hdr_capable(&self) -> Result<bool> {
    self.inner.is_hdr_capable()
  }

  /// Gets all supported refresh rates for this display.
  pub fn supported_refresh_rates(&self) -> Result<Vec<f32>> {
    self.inner.supported_refresh_rates()
  }

  /// Gets all supported resolutions for this display.
  pub fn supported_resolutions(&self) -> Result<Vec<(u32, u32)>> {
    self.inner.supported_resolutions()
  }

  /// Gets the ID of the device driving this display.
  pub fn device_id(&self) -> DisplayDeviceId {
    self.inner.device_id()
  }
}

/// State of a display device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayDeviceState {
  /// Device is active and can drive displays.
  Active,
  /// Device is present but inactive.
  Inactive,
  /// Device was disconnected.
  Disconnected,
}

/// Mirroring state of a display device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MirroringState {
  /// Not mirroring.
  None,
  /// This device is the source being mirrored.
  Source,
  /// This device is mirroring another.
  Mirror,
}

/// Display connection type for physical devices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayConnection {
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

/// Properties specific to physical display devices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalDeviceData {
  /// Device vendor name.
  pub vendor: Option<String>,
  /// Device model name.
  pub model: Option<String>,
  /// Device serial number.
  pub serial_number: Option<String>,
  /// Hardware identifier.
  pub hardware_id: Option<String>,
  /// EDID data if available.
  pub edid_data: Option<Vec<u8>>,
  /// Physical size in millimeters.
  pub physical_size_mm: Option<(u32, u32)>,
  /// Connection type.
  pub connection_type: Option<DisplayConnection>,
  /// Whether this is a built-in display.
  pub is_builtin: bool,
}

/// Properties specific to virtual display devices.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VirtualDeviceData {
  /// Virtual driver name.
  pub driver_name: Option<String>,
  /// Virtual adapter identifier.
  pub virtual_adapter_id: Option<String>,
}

/// Device-specific data separated by type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisplayDeviceData {
  /// Physical display device data.
  Physical(PhysicalDeviceData),
  /// Virtual display device data.
  Virtual(VirtualDeviceData),
}

/// Represents a display adapter/device (physical or virtual).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayDevice {
  pub(crate) inner: platform_impl::DisplayDevice,
}

impl DisplayDevice {
  /// Creates a new `DisplayDevice` from platform-specific data.
  #[must_use]
  pub(crate) fn from_platform_impl(
    inner: platform_impl::DisplayDevice,
  ) -> Self {
    Self { inner }
  }

  /// Gets the unique identifier for this display device.
  pub fn id(&self) -> DisplayDeviceId {
    self.inner.id()
  }

  /// Gets the device name.
  pub fn name(&self) -> Result<String> {
    self.inner.name()
  }

  /// Gets the current state of the device.
  pub fn state(&self) -> Result<DisplayDeviceState> {
    self.inner.state()
  }

  /// Gets the mirroring state of the device.
  pub fn mirroring_state(&self) -> Result<MirroringState> {
    self.inner.mirroring_state()
  }

  /// Gets the device-specific data.
  pub fn data(&self) -> Result<DisplayDeviceData> {
    self.inner.data()
  }

  /// Returns whether this is a physical device.
  pub fn is_physical(&self) -> Result<bool> {
    Ok(matches!(self.data()?, DisplayDeviceData::Physical(_)))
  }

  /// Returns whether this is a virtual device.
  pub fn is_virtual(&self) -> Result<bool> {
    Ok(matches!(self.data()?, DisplayDeviceData::Virtual(_)))
  }

  /// Returns whether this is a built-in device.
  ///
  /// Only physical devices can be built-in.
  pub fn is_builtin(&self) -> Result<bool> {
    match self.data()? {
      DisplayDeviceData::Physical(data) => Ok(data.is_builtin),
      DisplayDeviceData::Virtual(_) => Ok(false),
    }
  }
}

/// Gets all active displays.
///
/// Returns all displays that are currently active and available for use.
pub fn all_displays() -> Result<Vec<Display>> {
  Ok(
    platform_impl::all_displays()?
      .into_iter()
      .map(Display::from_platform_impl)
      .collect(),
  )
}

/// Gets all display devices.
///
/// Returns all display devices including active, inactive, and
/// disconnected ones.
pub fn all_display_devices() -> Result<Vec<DisplayDevice>> {
  Ok(
    platform_impl::all_display_devices()?
      .into_iter()
      .map(DisplayDevice::from_platform_impl)
      .collect(),
  )
}

/// Gets the display containing the specified point.
///
/// If no display contains the point, returns the primary display.
pub fn display_from_point(point: Point) -> Result<Display> {
  let display = platform_impl::display_from_point(point)?;
  Ok(Display::from_platform_impl(display))
}

/// Gets the primary display.
pub fn primary_display() -> Result<Display> {
  let display = platform_impl::primary_display()?;
  Ok(Display::from_platform_impl(display))
}
