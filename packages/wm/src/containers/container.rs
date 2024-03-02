use std::{
  cell::RefCell,
  collections::VecDeque,
  rc::{Rc, Weak},
};

use uuid::Uuid;

use crate::{
  monitors::Monitor,
  windows::{Window, WindowState},
  workspaces::Workspace,
};

use super::{ContainerType, RootContainer, SplitContainer};

#[derive(Debug)]
pub struct InnerContainer {
  pub id: Uuid,
  container_type: ContainerType,
  pub parent: Option<Weak<RefCell<Container>>>,
  pub children: VecDeque<Rc<RefCell<Container>>>,
  // TODO: Use vec of ids instead and a getter to get the actual containers.
  pub child_focus_order: VecDeque<Rc<RefCell<Container>>>,
}

impl InnerContainer {
  pub fn new(container_type: ContainerType) -> Self {
    Self {
      id: Uuid::new_v4(),
      container_type,
      parent: None,
      children: VecDeque::new(),
      child_focus_order: VecDeque::new(),
    }
  }
}

#[derive(Debug)]
pub enum Container {
  RootContainer(RootContainer),
  Monitor(Monitor),
  Workspace(Workspace),
  SplitContainer(SplitContainer),
  Window(Window),
}

impl Container {
  pub fn inner(&self) -> &InnerContainer {
    match self {
      Self::RootContainer(c) => &c.inner,
      Self::Monitor(c) => &c.inner,
      Self::Workspace(c) => &c.inner,
      Self::SplitContainer(c) => &c.inner,
      Self::Window(c) => &c.inner,
    }
  }

  pub fn inner_mut(&mut self) -> &mut InnerContainer {
    match self {
      Self::RootContainer(c) => &mut c.inner,
      Self::Monitor(c) => &mut c.inner,
      Self::Workspace(c) => &mut c.inner,
      Self::SplitContainer(c) => &mut c.inner,
      Self::Window(c) => &mut c.inner,
    }
  }

  // / Height of the container. Implementation varies by container type.
  // pub fn height(&self) -> u32 {
  //   match self.value {
  //     Self::Monitor(c) => c.height,
  //     Self::Workspace(c) => c.height,
  //     Self::SplitContainer(c) => c.height(),
  //     Self::Window(c) => c.height(),
  //     _ => 0,
  //   }
  // }

  // /// Width of the container. Implementation varies by container type.
  // pub fn width(&self) -> u32 {
  //   match self.value {
  //     Self::Monitor(c) => c.width,
  //     Self::Workspace(c) => c.width,
  //     Self::SplitContainer(c) => c.width(),
  //     Self::Window(c) => c.width(),
  //     _ => 0,
  //   }
  // }

  // /// X-coordinate of the container. Implementation varies by container type.
  // pub fn x(&self) -> u32 {
  //   match self.value {
  //     Self::Monitor(c) => c.x,
  //     Self::Workspace(c) => c.x,
  //     Self::SplitContainer(c) => c.x(),
  //     Self::Window(c) => c.x(),
  //     _ => 0,
  //   }
  // }

  // /// Y-coordinate of the container. Implementation varies by container type.
  // pub fn y(&self) -> u32 {
  //   match self.value {
  //     Self::Monitor(c) => c.y,
  //     Self::Workspace(c) => c.y,
  //     Self::SplitContainer(c) => c.y(),
  //     Self::Window(c) => c.y(),
  //     _ => 0,
  //   }
  // }

  // /// Whether the container can be tiled.
  // pub fn can_tile(&self) -> bool {
  //   match self.value {
  //     Self::Window(c) => c.state == WindowState::Tiling,
  //     _ => true,
  //   }
  // }

  // /// Unique identifier for the container.
  // fn id(&self) -> Uuid {
  //   self.inner().id
  // }

  // pub fn parent(&self) -> Option<Arc<Container>> {
  //   self.inner().parent.clone()
  // }

  // pub fn set_parent(&mut self, parent: Option<Arc<Container>>) {
  //   self.inner_mut().parent = parent;
  // }

  // pub fn children(&self) -> Vec<Arc<Container>> {
  //   self.inner().children.clone()
  // }

  // pub fn set_children(&self, children: Vec<Arc<Container>>) {
  //   self.inner().children = children;
  // }

  // /// Order of which child containers last had focus.
  // pub fn child_focus_order(&self) -> Vec<Arc<Container>> {
  //   self.inner().child_focus_order.clone()
  // }

  // pub fn set_child_focus_order(
  //   &mut self,
  //   child_focus_order: Vec<Arc<Container>>,
  // ) {
  //   self.inner().child_focus_order = child_focus_order;
  // }

  // /// Child container that last had focus.
  // pub fn last_focused_child(&self) -> Option<Arc<Container>> {
  //   self.child_focus_order().get(0).cloned()
  // }

  // /// Index of this container in parent's child focus order.
  // pub fn focus_index(&self) -> u32 {
  //   match &self.inner().parent {
  //     None => 0,
  //     Some(p) => p
  //       .child_focus_order()
  //       .iter()
  //       .position(|child| child.id() == self.id())
  //       .unwrap()
  //       .try_into()
  //       .unwrap(),
  //   }
  // }

  // pub fn parent_workspace(&self) -> Option<Rc<RefCell<Container>>> {}
  // pub fn parent_monitor(&self) -> Option<Rc<RefCell<Container>>> {}
}
