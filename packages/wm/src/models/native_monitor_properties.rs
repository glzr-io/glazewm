use wm_platform::{Display, Rect};

#[derive(Debug, Clone)]
pub struct NativeMonitorProperties {
  pub working_area: Rect,
  pub bounds: Rect,
  pub dpi: u32,
  pub scale_factor: f32,
}

impl NativeMonitorProperties {
  pub fn try_from(native_display: &Display) -> anyhow::Result<Self> {
    Ok(Self {
      working_area: native_display.working_area()?,
      bounds: native_display.bounds()?,
      dpi: native_display.dpi()?,
      scale_factor: native_display.scale_factor()?,
    })
  }
}
