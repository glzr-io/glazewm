use crate::containers::ContainerType;

use super::{ContainerVariant, InnerContainer};

#[derive(Debug)]
pub struct RootContainer {
  pub inner: InnerContainer,
  width: u32,
  height: u32,
  x: u32,
  y: u32,
}

impl RootContainer {
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

impl ContainerVariant for RootContainer {
  fn inner(&self) -> InnerContainer {
    self.inner
  }

  fn r#type(&self) -> ContainerType {
    ContainerType::RootContainer
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
