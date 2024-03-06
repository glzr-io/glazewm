use core::fmt;
use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::{Rc, Weak},
};

use uuid::Uuid;

// use super::{ContainerType, SplitContainer};

#[derive(Clone)]
pub struct RootContainerRef(Rc<RefCell<RootContainer>>);

#[derive(Debug)]
pub struct RootContainer {
  pub parent: Option<ContainerRef>,
  pub children: Vec<ContainerRef>,
  pub val: i32,
}

/// A reference to a `Container`.
///
/// Internally, this uses reference counting for lifetime tracking and
/// `std::cell::RefCell` for interior mutability.
///
/// **Note:** Cloning a `ContainerRef` only increments a reference count.
#[derive(Clone, Debug)]
pub enum ContainerRef {
  RootContainer(RootContainerRef),
  // Monitor(MonitorRef),
  // Workspace(WorkspaceRef),
  // SplitContainer(SplitContainerRef),
  // Window(WindowRef),
}

pub trait CommonContainer {
  /// Return a reference to the parent node, unless this node is the root
  /// of the tree.
  ///
  /// # Panics
  ///
  /// Panics if the node is currently mutability borrowed.
  fn parent(&self) -> Option<ContainerRef>;
  fn set_parent(&self, parent: ContainerRef);
  fn grandparent(&self) -> Option<ContainerRef>;

  fn insert_child(&self, child: ContainerRef);
}

impl RootContainerRef {
  pub fn new(val: i32) -> Self {
    let root = RootContainer {
      parent: None,
      children: Vec::new(),
      val,
    };

    Self(Rc::new(RefCell::new(root)))
  }

  pub fn set_val(&self, val: i32) {
    self.0.borrow_mut().val = val
  }
}

impl CommonContainer for RootContainerRef {
  fn grandparent(&self) -> Option<ContainerRef> {
    self.parent()?.common().parent()
  }

  fn parent(&self) -> Option<ContainerRef> {
    self.0.borrow().parent.clone()
  }

  fn insert_child(&self, child: ContainerRef) {
    self.0.borrow_mut().children.push(child.clone());
    child
      .common()
      .set_parent(ContainerRef::RootContainer(self.clone()))
  }

  fn set_parent(&self, parent: ContainerRef) {
    self.0.borrow_mut().parent = Some(parent)
  }
}

impl ContainerRef {
  fn common(&self) -> &dyn CommonContainer {
    match self {
      ContainerRef::RootContainer(c) => c,
    }
  }
}

impl fmt::Debug for RootContainerRef {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(&self.0.borrow().val, f)
  }
}
