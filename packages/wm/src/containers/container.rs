use core::fmt;
use std::{
  cell::{Ref, RefCell, RefMut},
  collections::VecDeque,
  rc::{Rc, Weak},
};

use uuid::Uuid;

use super::{ContainerType, RootContainer, SplitContainer};

/// A reference to a `Container`.
///
/// Internally, this uses reference counting for lifetime tracking and
/// `std::cell::RefCell` for interior mutability.
///
/// **Note:** Cloning a `ContainerRef` only increments a reference count.
pub struct ContainerRef(Rc<RefCell<Container>>);

struct Container {
  parent: Option<Weak<RefCell<Container>>>,
  children: Vec<Rc<RefCell<Container>>>,
  data: ContainerData,
}

#[derive(Debug)]
pub enum ContainerData {
  RootContainer,
  Monitor,
  Workspace,
  SplitContainer,
  Window,
  // RootContainer(RootContainer),
  // Monitor(Monitor),
  // Workspace(Workspace),
  // SplitContainer(SplitContainer),
  // Window(Window),
}

/// Cloning a `ContainerRef` only increments a reference count. It does not
/// copy the data.
impl Clone for ContainerRef {
  fn clone(&self) -> ContainerRef {
    ContainerRef(self.0.clone())
  }
}

impl fmt::Debug for ContainerRef {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(&*self.borrow(), f)
  }
}

impl ContainerRef {
  /// Create a new node from its associated data.
  pub fn new(data: ContainerData) -> ContainerRef {
    ContainerRef(Rc::new(RefCell::new(Container {
      parent: None,
      children: Vec::new(),
      data,
    })))
  }

  /// Return a reference to the parent node, unless this node is the root
  /// of the tree.
  ///
  /// # Panics
  ///
  /// Panics if the node is currently mutability borrowed.
  pub fn parent(&self) -> Option<ContainerRef> {
    Some(ContainerRef(self.0.borrow().parent.as_ref()?.upgrade()?))
  }

  /// Return a shared reference to this node’s data
  ///
  /// # Panics
  ///
  /// Panics if the node is currently mutability borrowed.
  pub fn borrow(&self) -> Ref<ContainerData> {
    Ref {
      _ref: self.0.borrow(),
    }
  }

  /// Return a unique/mutable reference to this node’s data
  ///
  /// # Panics
  ///
  /// Panics if the node is currently borrowed.
  pub fn borrow_mut(&self) -> RefMut<ContainerData> {
    RefMut {
      _ref: self.0.borrow_mut(),
    }
  }
}
