use std::sync::Arc;

use uuid::Uuid;

use crate::{
  common::RectDelta,
  containers::{Container, ContainerType},
  monitors::Monitor,
};

pub struct Workspace {
  id: Uuid,
  parent: Monitor,
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
    parent: Monitor,
  ) -> Self {
    Self {
      id: Uuid::new_v4(),
      name,
      display_name,
      keep_alive,
      outer_gaps,
      parent,
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

  fn parent(&self) -> Arc<dyn Container> {
    todo!()
  }

  fn children(&self) -> Vec<Arc<dyn Container>> {
    todo!()
  }

  fn child_focus_order(&self) -> Vec<Arc<dyn Container>> {
    todo!()
  }

  fn set_parent(&mut self, parent: std::sync::Arc<dyn Container>) -> () {
    todo!()
  }

  fn set_children(
    &self,
    children: Vec<std::sync::Arc<dyn Container>>,
  ) -> () {
    todo!()
  }

  fn set_child_focus_order(
    &self,
    child_focus_order: Vec<std::sync::Arc<dyn Container>>,
  ) -> () {
    todo!()
  }
}
