use enum_dispatch::enum_dispatch;

use crate::{
  monitors::Monitor,
  windows::{NonTilingWindow, TilingWindow},
  workspaces::Workspace,
};

use super::{traits::TilingContainer, RootContainer, SplitContainer};

/// A reference to a container.
///
/// Internally, this uses reference counting for lifetime tracking and
/// `std::cell::RefCell` for interior mutability.
///
/// **Note:** Cloning a `ContainerRef` only increments a reference count.
#[derive(Clone, Debug)]
#[enum_dispatch(CommonContainer)]
pub enum Container {
  Root(RootContainer),
  Monitor(Monitor),
  Workspace(Workspace),
  Split(SplitContainer),
  TilingWindow(TilingWindow),
  NonTilingWindow(NonTilingWindow),
}

impl Container {
  pub fn as_tiling(&self) -> &dyn TilingContainer {
    match self {
      Container::Root(c) => c,
      Container::Monitor(c) => c,
      // ContainerRef::Workspace(c) => c,
      // ContainerRef::SplitContainer(c) => c,
      // ContainerRef::Window(c) => c,
      _ => todo!(),
    }
  }
}

// impl CommonContainer for ContainerRef {
//   fn id(&self) -> Uuid {
//     self.as_common().id()
//   }

//   fn r#type(&self) -> ContainerType {
//     self.as_common().r#type()
//   }

//   fn borrow_parent(&self) -> Ref<'_, Option<ContainerRef>> {
//     self.as_common().borrow_parent()
//   }

//   fn borrow_parent_mut(&self) -> RefMut<'_, Option<ContainerRef>> {
//     self.as_common().borrow_parent_mut()
//   }

//   fn parent(&self) -> Option<ContainerRef> {
//     self.as_common().parent()
//   }
// }
