use crate::{
  common::platform::NativeMonitor,
  containers::{ContainerType, InnerContainer},
};

#[derive(Debug)]
pub struct Monitor {
  pub inner: InnerContainer,
  pub device_name: String,
  pub width: u32,
  pub height: u32,
  pub x: u32,
  pub y: u32,
}

impl Monitor {
  pub fn new(native_monitor: NativeMonitor) -> Self {
    Self {
      inner: InnerContainer::new(ContainerType::Monitor),
      device_name: native_monitor.device_name,
      width: native_monitor.width,
      height: native_monitor.height,
      x: native_monitor.x,
      y: native_monitor.y,
    }
  }
}
