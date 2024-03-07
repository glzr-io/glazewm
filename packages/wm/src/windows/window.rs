use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug)]
pub struct WindowRef(Rc<RefCell<Window>>);

// TODO: Consider renaming to `TilingWindow` and splitting out `NonTilingWindow`.
#[derive(Debug)]
pub struct Window {
  width: u32,
  height: u32,
  x: u32,
  y: u32,
}

impl Window {
  pub fn new() -> Self {
    Self {
      width: 0,
      height: 0,
      x: 0,
      y: 0,
    }
  }
}
