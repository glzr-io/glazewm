use wm_platform::{Display, Rect};
#[cfg(target_os = "windows")]
use wm_platform::{DisplayDeviceExtWindows, DisplayExtWindows};

#[derive(Debug, Clone)]
pub struct NativeMonitorProperties {
  #[cfg(target_os = "macos")]
  pub device_uuid: String,
  #[cfg(target_os = "windows")]
  pub handle: isize,
  #[cfg(target_os = "windows")]
  pub hardware_id: Option<String>,
  #[cfg(target_os = "windows")]
  pub device_path: Option<String>,
  pub working_area: Rect,
  pub bounds: Rect,
  pub dpi: u32,
  pub scale_factor: f32,
}

impl NativeMonitorProperties {
  pub fn try_from(native_display: &Display) -> anyhow::Result<Self> {
    let display_device = native_display.main_device()?;

    Ok(Self {
      #[cfg(target_os = "macos")]
      device_uuid: display_device.id().0,
      #[cfg(target_os = "windows")]
      handle: native_display.hmonitor().0,
      #[cfg(target_os = "windows")]
      hardware_id: display_device.hardware_id(),
      #[cfg(target_os = "windows")]
      device_path: display_device.device_path(),
      working_area: native_display.working_area()?,
      bounds: native_display.bounds()?,
      dpi: native_display.dpi()?,
      scale_factor: native_display.scale_factor()?,
    })
  }
}
