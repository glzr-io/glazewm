use uuid::Uuid;

use crate::containers::{Container, ContainerType};

pub struct Workspace {
  id: Uuid,
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
      id: Uuid::new_v4(),
      name,
      display_name,
      keep_alive,
      outer_gaps,
    }
  }
}

impl Container for Workspace {
  fn id(&self) -> Uuid {
    self.id
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

  fn parent(&self) -> Box<dyn Container> {
    todo!()
  }

  fn children(&self) -> Vec<Box<dyn Container>> {
    todo!()
  }

  fn child_focus_order(&self) -> Vec<Box<dyn Container>> {
    todo!()
  }
}
