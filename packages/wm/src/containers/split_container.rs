use crate::containers::ContainerType;

use super::InnerContainer;

#[derive(Debug)]
pub struct SplitContainer {
  pub inner: InnerContainer,
  width: u32,
  height: u32,
  x: u32,
  y: u32,
}

impl SplitContainer {
  pub fn new() -> Self {
    Self {
      inner: InnerContainer::new(ContainerType::SplitContainer),
      width: 0,
      height: 0,
      x: 0,
      y: 0,
    }
  }
}
