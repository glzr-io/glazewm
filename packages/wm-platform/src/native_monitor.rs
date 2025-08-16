use wm_common::{Point, Rect};

use crate::{platform_impl, Result};

/// State of a monitor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MonitorState {
  /// Monitor is active and available for display.
  Active,
  /// Monitor is mirroring another display.
  Mirroring,
  /// Monitor is present but inactive.
  Inactive,
}

/// Cross-platform monitor representation.
///
/// This struct provides a unified interface to monitor information
/// across different platforms. It delegates to platform-specific
/// implementations internally.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeMonitor {
  pub(crate) inner: platform_impl::NativeMonitor,
}

impl NativeMonitor {
  /// Creates a new `NativeMonitor` from platform-specific data.
  #[must_use]
  pub(crate) fn from_platform_impl(
    inner: platform_impl::NativeMonitor,
  ) -> Self {
    Self { inner }
  }

  /// Gets the device name of the monitor.
  pub fn device_name(&self) -> Result<String> {
    self.inner.device_name()
  }

  /// Gets the hardware identifier for the monitor.
  pub fn hardware_id(&self) -> Result<Option<String>> {
    self.inner.hardware_id()
  }

  /// Gets the full bounds rectangle of the monitor.
  pub fn rect(&self) -> Result<Rect> {
    self.inner.rect()
  }

  /// Gets the working area rectangle (excluding system UI).
  pub fn working_rect(&self) -> Result<Rect> {
    self.inner.working_rect()
  }

  /// Gets the DPI for the monitor.
  pub fn dpi(&self) -> Result<u32> {
    self.inner.dpi()
  }

  /// Gets the scale factor for the monitor.
  pub fn scale_factor(&self) -> Result<f32> {
    self.inner.scale_factor()
  }

  /// Gets the current state of the monitor.
  pub fn state(&self) -> Result<MonitorState> {
    self.inner.state()
  }

  /// Returns whether this is the primary monitor.
  pub fn is_primary(&self) -> Result<bool> {
    self.inner.is_primary()
  }
}

/// Gets all monitors, including active, mirroring, and inactive ones.
pub fn all_monitors() -> Result<Vec<NativeMonitor>> {
  Ok(
    platform_impl::all_monitors()?
      .into_iter()
      .map(NativeMonitor::from_platform_impl)
      .collect(),
  )
}

/// Gets only active monitors (excludes mirroring and inactive).
pub fn active_monitors() -> Result<Vec<NativeMonitor>> {
  Ok(
    all_monitors()?
      .into_iter()
      .filter(|monitor| {
        matches!(monitor.state(), Ok(MonitorState::Active))
      })
      .collect(),
  )
}

/// Gets the monitor containing the specified point.
///
/// If no monitor contains the point, returns the primary monitor.
pub fn monitor_from_point(point: Point) -> Result<NativeMonitor> {
  let monitor = platform_impl::monitor_from_point(point)?;
  Ok(monitor.into())
}

/// Gets the primary monitor.
pub fn primary_monitor() -> Result<NativeMonitor> {
  let monitor = platform_impl::primary_monitor()?;
  Ok(monitor.into())
}
