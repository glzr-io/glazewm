use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct TilingWindowRef(Rc<RefCell<TilingWindow>>);

#[derive(Debug)]
pub struct TilingWindow {
  width: u32,
  height: u32,
  x: u32,
  y: u32,
}

impl TilingWindow {
  pub fn new() -> Self {
    Self {
      width: 0,
      height: 0,
      x: 0,
      y: 0,
    }
  }
}
