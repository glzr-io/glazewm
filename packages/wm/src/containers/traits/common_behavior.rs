use std::cell::{Ref, RefMut};

use enum_dispatch::enum_dispatch;
use uuid::Uuid;

use crate::{
  containers::{
    Container, ContainerType, RootContainer, SplitContainer,
    TilingContainer,
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

  fn self_as_container(&self) -> Container;

  fn borrow_parent(&self) -> Ref<'_, Option<TilingContainer>>;

  fn borrow_parent_mut(&self) -> RefMut<'_, Option<TilingContainer>>;

  fn borrow_children(&self) -> Ref<'_, Vec<Container>>;

  fn borrow_children_mut(&self) -> RefMut<'_, Vec<Container>>;

  /// Returns a reference to the parent container, unless this container is
  /// the root.
  ///
  /// # Panics
  ///
  /// Panics if the container is currently mutably borrowed.
  fn parent(&self) -> Option<TilingContainer> {
    self.borrow_parent().clone()
  }

  fn children(&self) -> Vec<Container> {
    self.borrow_children().clone()
  }

  fn descendants(&self) -> Descendants {
    Descendants {
      stack: self.children(),
    }
  }

  fn siblings(&self) -> Siblings {
    Siblings(Some(self.self_as_container()))
  }

  fn ancestors(&self) -> Ancestors {
    Ancestors(self.parent())
  }
}

/// Helper macro for implementing iterators that return `Option<Container>`.
#[macro_export]
macro_rules! impl_container_iterator {
  ($name: ident, $next: expr) => {
    impl Iterator for $name {
      type Item = Container;

      fn next(&mut self) -> Option<Container> {
        match self.0.take() {
          Some(container) => {
            self.0 = $next(&container);
            Some(container)
          }
          None => None,
        }
      }
    }
  };
}

/// Helper macro for implementing iterators that return `Option<TilingContainer>`.
#[macro_export]
macro_rules! impl_tiling_container_iterator {
  ($name: ident, $next: expr) => {
    impl Iterator for $name {
      type Item = TilingContainer;

      fn next(&mut self) -> Option<TilingContainer> {
        match self.0.take() {
          Some(container) => {
            self.0 = $next(&container);
            Some(container)
          }
          None => None,
        }
      }
    }
  };
}

/// An iterator over ancestors of a given container.
pub struct Ancestors(Option<TilingContainer>);
impl_tiling_container_iterator!(
  Ancestors,
  |container: &TilingContainer| { container.parent() }
);

/// An iterator over siblings of a given container.
pub struct Siblings(Option<Container>);
impl_container_iterator!(Siblings, |container: &Container| {
  container.parent().and_then(|parent| {
    parent
      .children()
      .into_iter()
      .find(|child| child != container)
  })
});

/// An iterator over descendants of a given container.
pub struct Descendants {
  stack: Vec<Container>,
}

impl Iterator for Descendants {
  type Item = Container;

  fn next(&mut self) -> Option<Container> {
    while let Some(container) = self.stack.pop() {
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

      fn self_as_container(&self) -> Container {
        self.clone().into()
      }

      fn borrow_parent(&self) -> Ref<'_, Option<TilingContainer>> {
        Ref::map(self.0.borrow(), |c| &c.parent)
      }

      fn borrow_parent_mut(&self) -> RefMut<'_, Option<TilingContainer>> {
        RefMut::map(self.0.borrow_mut(), |c| &mut c.parent)
      }

      fn borrow_children(&self) -> Ref<'_, Vec<Container>> {
        Ref::map(self.0.borrow(), |c| &c.children)
      }

      fn borrow_children_mut(&self) -> RefMut<'_, Vec<Container>> {
        RefMut::map(self.0.borrow_mut(), |c| &mut c.children)
      }
    }
  };
}
