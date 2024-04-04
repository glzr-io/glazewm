use std::{
  cell::{Ref, RefMut},
  collections::VecDeque,
};

use enum_dispatch::enum_dispatch;
use uuid::Uuid;

use crate::{
  containers::{
    Container, ContainerType, RootContainer, SplitContainer,
    TilingContainer, WindowContainer,
  },
  monitors::Monitor,
  windows::{NonTilingWindow, TilingWindow},
  workspaces::Workspace,
};

#[enum_dispatch]
pub trait CommonBehavior {
  /// A unique identifier for the container.
  fn id(&self) -> Uuid;

  /// Derived container type (eg. `ContainerType::Monitor`).
  fn r#type(&self) -> ContainerType;

  fn as_container(&self) -> Container;

  fn borrow_parent(&self) -> Ref<'_, Option<TilingContainer>>;

  fn borrow_parent_mut(&self) -> RefMut<'_, Option<TilingContainer>>;

  fn borrow_children(&self) -> Ref<'_, VecDeque<Container>>;

  fn borrow_children_mut(&self) -> RefMut<'_, VecDeque<Container>>;

  /// Gets the parent container, unless this container is the root.
  fn parent(&self) -> Option<TilingContainer> {
    self.borrow_parent().clone()
  }

  /// Direct children of this container.
  fn children(&self) -> VecDeque<Container> {
    self.borrow_children().clone()
  }

  /// Whether this container is detached from the tree (ie. it does not
  /// have a parent).
  fn is_detached(&self) -> bool {
    self.borrow_parent().as_ref().is_none()
  }

  /// Index of this container amongst its siblings.
  fn index(&self) -> Option<usize> {
    self.borrow_parent().as_ref().and_then(|parent| {
      parent
        .children()
        .iter()
        .position(|child| child.id() == self.id())
    })
  }

  fn descendants(&self) -> Descendants {
    Descendants {
      stack: self.children(),
    }
  }

  fn self_and_descendants(&self) -> Descendants {
    let mut stack = self.children();
    stack.push_front(self.as_container());
    Descendants { stack }
  }

  fn siblings(&self) -> Box<dyn Iterator<Item = Container> + '_> {
    Box::new(
      self
        .parent()
        .into_iter()
        .flat_map(|parent| parent.children())
        .filter(move |sibling| sibling.id() != self.id()),
    )
  }

  fn tiling_siblings(
    &self,
  ) -> Box<dyn Iterator<Item = TilingContainer> + '_> {
    Box::new(
      self
        .siblings()
        .filter_map(|container| container.try_into().ok()),
    )
  }

  fn ancestors(&self) -> Ancestors {
    Ancestors {
      start: self.parent().map(|c| c.into()),
    }
  }

  fn self_and_ancestors(&self) -> Ancestors {
    Ancestors {
      start: Some(self.as_container()),
    }
  }
}

/// An iterator over ancestors of a given container.
pub struct Ancestors {
  start: Option<Container>,
}

impl Iterator for Ancestors {
  type Item = Container;

  fn next(&mut self) -> Option<Container> {
    self.start.take().map(|container| {
      self.start = container.parent().map(|c| c.into());
      container
    })
  }
}

/// An iterator over siblings of a given container.
// pub struct Siblings {
//   ignore: Container,
//   current: Option<Container>,
// }

// impl Iterator for Siblings {
//   type Item = Container;

//   fn next(&mut self) -> Option<Container> {
//     self.current.take().map(|container| {
//       self.ignore = container.parent().map(|c| c.into());
//       container
//     })
//   }
// }

// impl_container_iterator!(Siblings, |container: &Container| {
//   println!("siblings: {:?}", container.id());
//   container.parent().and_then(|parent| {
//     parent
//       .children()
//       .into_iter()
//       .find(|child| child != container)
//   })
// });

/// An iterator over descendants of a given container.
pub struct Descendants {
  stack: VecDeque<Container>,
}

impl Iterator for Descendants {
  type Item = Container;

  fn next(&mut self) -> Option<Container> {
    while let Some(container) = self.stack.pop_front() {
      self.stack.extend(container.children());
      return Some(container);
    }
    None
  }
}

/// Implements the `CommonBehavior` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with an `id` and a `parent` field.
#[macro_export]
macro_rules! impl_common_behavior {
  ($struct_name:ident, $container_type:expr) => {
    impl CommonBehavior for $struct_name {
      fn id(&self) -> Uuid {
        self.0.borrow().id
      }

      fn r#type(&self) -> ContainerType {
        $container_type
      }

      fn as_container(&self) -> Container {
        self.clone().into()
      }

      fn borrow_parent(&self) -> Ref<'_, Option<TilingContainer>> {
        Ref::map(self.0.borrow(), |c| &c.parent)
      }

      fn borrow_parent_mut(&self) -> RefMut<'_, Option<TilingContainer>> {
        RefMut::map(self.0.borrow_mut(), |c| &mut c.parent)
      }

      fn borrow_children(&self) -> Ref<'_, VecDeque<Container>> {
        Ref::map(self.0.borrow(), |c| &c.children)
      }

      fn borrow_children_mut(&self) -> RefMut<'_, VecDeque<Container>> {
        RefMut::map(self.0.borrow_mut(), |c| &mut c.children)
      }
    }
  };
}
