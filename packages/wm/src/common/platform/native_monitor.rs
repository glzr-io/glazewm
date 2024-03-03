pub struct NativeMonitor {
  pub device_name: String,
  pub width: u32,
  pub height: u32,
  pub x: u32,
  pub y: u32,
}

impl NativeMonitor {
  pub fn new(
    device_name: String,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
  ) -> Self {
    Self {
      device_name,
      width,
      height,
      x,
      y,
    }
  }
}
