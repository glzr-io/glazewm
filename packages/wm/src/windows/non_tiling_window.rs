use std::{cell::RefCell, rc::Rc};

use crate::workspaces::WorkspaceRef;

#[derive(Clone, Debug)]
pub struct NonTilingWindowRef(Rc<RefCell<NonTilingWindow>>);

#[derive(Debug)]
pub struct NonTilingWindow {
  parent: Option<WorkspaceRef>,
  width: u32,
  height: u32,
  x: u32,
  y: u32,
}

impl NonTilingWindow {
  pub fn new() -> Self {
    Self {
      parent: None,
      width: 0,
      height: 0,
      x: 0,
      y: 0,
    }
  }
}
