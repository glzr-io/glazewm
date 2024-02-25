use crate::containers::{ContainerType, ContainerVariant, InnerContainer};

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
      inner: InnerContainer::new(None, vec![]),
      device_name,
      width,
      height,
      x,
      y,
    }
  }
}

impl ContainerVariant for Monitor {
  fn inner(&self) -> InnerContainer {
    self.inner
  }

  fn r#type(&self) -> ContainerType {
    ContainerType::Monitor
  }

  fn height(&self) -> u32 {
    self.height
  }

  fn width(&self) -> u32 {
    self.width
  }

  fn x(&self) -> u32 {
    self.x
  }

  fn y(&self) -> u32 {
    self.y
  }
}
