use crate::containers::InnerContainer;

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
