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

  fn borrow_parent(&self) -> Ref<'_, Option<TilingContainer>>;

  fn borrow_parent_mut(&self) -> RefMut<'_, Option<TilingContainer>>;

  /// Returns a reference to the parent container, unless this container is
  /// the root.
  ///
  /// # Panics
  ///
  /// Panics if the container is currently mutably borrowed.
  fn parent(&self) -> Option<TilingContainer> {
    self.borrow_parent().clone()
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

      fn borrow_parent(&self) -> Ref<'_, Option<TilingContainer>> {
        Ref::map(self.0.borrow(), |c| &c.parent)
      }

      fn borrow_parent_mut(&self) -> RefMut<'_, Option<TilingContainer>> {
        RefMut::map(self.0.borrow_mut(), |c| &mut c.parent)
      }
    }
  };
}
