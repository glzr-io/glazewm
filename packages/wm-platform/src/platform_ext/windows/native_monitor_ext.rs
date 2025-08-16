/// Windows-specific extensions for `NativeMonitor`.
///
/// This trait provides access to platform-specific functionality
/// that is only available on Windows.
#[cfg(target_os = "windows")]
pub trait NativeMonitorExtWindows {
  /// Gets the Windows monitor handle.
  fn hmonitor(&self) -> HMONITOR;

  /// Gets the device path for the monitor.
  ///
  /// This is the DOS device path used by the display driver.
  ///
  /// # Platform-specific
  /// This method is only available on Windows.
  fn device_path(&self) -> Result<Option<String>>;
}

#[cfg(target_os = "windows")]
impl NativeMonitorExtWindows for NativeMonitor {
  fn hmonitor(&self) -> HMONITOR {
    HMONITOR(self.inner.handle)
  }

  fn device_path(&self) -> Result<Option<String>> {
    Ok(None) // TODO: Implement proper device path access
  }
}
