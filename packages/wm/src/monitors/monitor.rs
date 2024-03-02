use crate::containers::{ContainerType, InnerContainer};

#[derive(Debug)]
pub struct Monitor {
  pub inner: InnerContainer,
  device_name: String,
  width: u32,
  height: u32,
  x: u32,
  y: u32,
}

impl Monitor {
  pub fn new(
    device_name: String,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
  ) -> Self {
    Self {
      inner: InnerContainer::new(ContainerType::Monitor),
      device_name,
      width,
      height,
      x,
      y,
    }
  }
}
