use crate::{
  common::RectDelta,
  containers::{ContainerType, InnerContainer},
};

#[derive(Debug)]
pub struct Workspace {
  pub inner: InnerContainer,
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
      inner: InnerContainer::new(ContainerType::Workspace),
      name,
      display_name,
      keep_alive,
      outer_gaps,
    }
  }
}
