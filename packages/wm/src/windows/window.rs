use crate::containers::{ContainerType, InnerContainer};

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
      inner: InnerContainer::new(ContainerType::Window),
      width: 0,
      height: 0,
      x: 0,
      y: 0,
    }
  }
}
