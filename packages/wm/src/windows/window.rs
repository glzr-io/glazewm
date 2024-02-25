use crate::containers::{ContainerType, ContainerVariant, InnerContainer};

#[derive(Debug)]
pub struct Window {
  pub inner: InnerContainer,
  width: u32,
  height: u32,
  x: u32,
  y: u32,
}

impl Window {
  pub fn new() -> Self {
    Self {
      inner: InnerContainer::new(None, vec![]),
      width: 0,
      height: 0,
      x: 0,
      y: 0,
    }
  }
}

impl ContainerVariant for Window {
  fn inner(&self) -> InnerContainer {
    self.inner
  }

  fn r#type(&self) -> ContainerType {
    ContainerType::Window
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
