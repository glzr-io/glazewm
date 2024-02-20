use std::sync::Arc;

use uuid::Uuid;

use crate::{
  containers::{Container, ContainerType, RootContainer},
  workspaces::Workspace,
};

pub struct Monitor {
  id: Uuid,
  parent: Arc<RootContainer>,
  children: Vec<Arc<Workspace>>,
  child_focus_order: Vec<Arc<Workspace>>,
  device_name: String,
  width: u32,
  height: u32,
  x: u32,
  y: u32,
}

impl Monitor {
  pub fn new(
    parent: RootContainer,
    device_name: String,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
  ) -> Self {
    Self {
      id: Uuid::new_v4(),
      parent: Arc::new(parent),
      children: Vec::new(),
      child_focus_order: Vec::new(),
      device_name,
      width,
      height,
      x,
      y,
    }
  }
}

impl Container for Monitor {
  fn id(&self) -> Uuid {
    self.id
  }

  fn r#type(&self) -> ContainerType {
    ContainerType::Monitor
  }

  fn height(&self) -> u32 {
    self.height
  }

  fn width(&self) -> u32 {
    self.width
  }

  fn x(&self) -> u32 {
    self.x
  }

  fn y(&self) -> u32 {
    self.y
  }

  fn parent(&self) -> Arc<dyn Container> {
    self.parent
  }

  fn children(&self) -> Vec<Arc<dyn Container>> {
    self.children
  }

  fn child_focus_order(&self) -> Vec<Arc<dyn Container>> {
    self.child_focus_order
  }

  fn set_parent(&mut self, parent: Arc<dyn Container>) -> () {
    self.parent = parent
  }

  fn set_children(&self, children: Vec<Arc<dyn Container>>) -> () {
    self.children = children
  }

  fn set_child_focus_order(
    &self,
    child_focus_order: Vec<Arc<dyn Container>>,
  ) -> () {
    self.child_focus_order = child_focus_order
  }
}
