use std::{cell::RefCell, rc::Rc};

use crate::common::TilingDirection;

#[derive(Clone, Debug)]
pub struct SplitContainerRef(Rc<RefCell<SplitContainer>>);

#[derive(Debug)]
pub struct SplitContainer {
  tiling_direction: TilingDirection,
}

impl SplitContainer {
  pub fn new(tiling_direction: TilingDirection) -> Self {
    Self { tiling_direction }
  }
}
