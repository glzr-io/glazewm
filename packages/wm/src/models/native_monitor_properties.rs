use wm_platform::{
  Display, DisplayDeviceExtWindows, DisplayExtWindows, Rect,
};

#[derive(Debug, Clone)]
pub struct NativeMonitorProperties {
  pub handle: isize,
  pub hardware_id: Option<String>,
  pub device_path: Option<String>,
  pub device_name: String,
  pub working_area: Rect,
  pub bounds: Rect,
  pub dpi: u32,
  pub scale_factor: f32,
}

impl NativeMonitorProperties {
  pub fn try_from(native_display: &Display) -> anyhow::Result<Self> {
    let display_device = native_display.main_device()?;

    Ok(Self {
      handle: native_display.hmonitor().0,
      hardware_id: display_device.hardware_id(),
      device_path: display_device.device_path(),
      device_name: native_display.name()?,
      working_area: native_display.working_area()?,
      bounds: native_display.bounds()?,
      dpi: native_display.dpi()?,
      scale_factor: native_display.scale_factor()?,
    })
  }
}
