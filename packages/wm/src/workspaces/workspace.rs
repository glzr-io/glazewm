use std::{cell::RefCell, rc::Rc};

use crate::common::RectDelta;

#[derive(Clone, Debug)]
pub struct WorkspaceRef(Rc<RefCell<Workspace>>);

#[derive(Debug)]
pub struct Workspace {
  name: String,
  display_name: String,
  keep_alive: bool,
  outer_gaps: RectDelta,
}

impl Workspace {
  pub fn new(
    name: String,
    display_name: String,
    keep_alive: bool,
    outer_gaps: RectDelta,
  ) -> Self {
    Self {
      name,
      display_name,
      keep_alive,
      outer_gaps,
    }
  }
}
