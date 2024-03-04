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

// TODO: Consider renaming to ContainerRelations and removing `container_type`.
// Instead create a `CommonContainer` trait that all containers implement (including
// the `Container` enum itself). Then change `relations` to be private and only
// accessible through `CommonContainer` trait methods.
// pub trait CommonContainer {
//   id() -> Uuid;
//   r#type() -> ContainerType;
//   relations() -> ContainerRelations;
// }

// It's possible to get `InnerContainer` to point to the outer via `Rc::new_cyclic`
// eg. https://stackoverflow.com/questions/67525645/constructor-for-a-structure-with-weak-reference-to-its-owner

// ----------

// If RcTree was used instead, could do:
// Could do `borrow_monitor` to get derived type.
// Ref: https://github.com/GNOME/librsvg/blob/d3c3269fa54c9b3fb60fcfdba29ff97e2029f600/rsvg/src/node.rs#L223

/// Strong reference to a container.
pub type Container = rctree::Node<ContainerData>;

/// Weak reference to a container.
pub type WeakContainer = rctree::WeakNode<ContainerData>;

#[derive(Debug)]
pub enum ContainerData {
  RootContainer(RootContainer),
  Monitor(Monitor),
  Workspace(Workspace),
  SplitContainer(SplitContainer),
  Window(Window),
}

trait CommonContainer {
  fn id(&self) -> Uuid;

  fn r#type(&self) -> ContainerType;

  fn child_focus_order(&self) -> VecDeque<Uuid>;
  fn child_focus_order_mut(&self) -> VecDeque<Uuid>;

  fn insert_child(&self, target_index: usize, child: Container) {
    child.inner_mut().parent = Some(Weak::new(RefCell::new(self)));

    self
      .children
      .insert(target_index, Rc::new(RefCell::new(child)));

    self.child_focus_order.push_front(child.id());
  }

  fn remove_child(&mut self, child_id: Uuid) {
    if let Some(index) = self
      .children
      .iter()
      .position(|child| child.borrow().id() == child_id)
    {
      self.children.remove(index);
      self.child_focus_order.retain(|id| *id != child_id);
    }
  }

  fn is_detached(&self) -> bool {
    self.parent.is_none()
  }

  fn has_children(&self) -> bool {
    !self.children.is_empty()
  }

  fn siblings(&self) -> Vec<Rc<RefCell<Container>>> {
    if let Some(parent) = self.parent.as_ref() {
      let parent = parent.upgrade().unwrap();
      let parent = parent.borrow();
      parent.children.clone().into()
    } else {
      vec![]
    }
  }
}
