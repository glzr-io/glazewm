use super::{ContainerType, InnerContainer};

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
      inner: InnerContainer::new(ContainerType::RootContainer),
      width: 0,
      height: 0,
      x: 0,
      y: 0,
    }
  }
}
