use crate::{
  common::RectDelta,
  containers::{ContainerType, ContainerVariant, InnerContainer},
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
      inner: InnerContainer::new(None, vec![]),
      name,
      display_name,
      keep_alive,
      outer_gaps,
    }
  }
}

impl ContainerVariant for Workspace {
  fn inner(&self) -> InnerContainer {
    self.inner
  }

  fn r#type(&self) -> ContainerType {
    ContainerType::Workspace
  }

  fn height(&self) -> u32 {
    todo!()
  }

  fn width(&self) -> u32 {
    todo!()
  }

  fn x(&self) -> u32 {
    todo!()
  }

  fn y(&self) -> u32 {
    todo!()
  }
}
