use crate::{
  monitors::MonitorRef,
  windows::{NonTilingWindowRef, TilingWindowRef},
  workspaces::WorkspaceRef,
};

use super::{
  traits::{CommonContainer, TilingContainer},
  RootContainerRef, SplitContainerRef,
};

/// A reference to a container.
///
/// Internally, this uses reference counting for lifetime tracking and
/// `std::cell::RefCell` for interior mutability.
///
/// **Note:** Cloning a `ContainerRef` only increments a reference count.
#[derive(Clone, Debug)]
pub enum ContainerRef {
  RootContainer(RootContainerRef),
  Monitor(MonitorRef),
  Workspace(WorkspaceRef),
  SplitContainer(SplitContainerRef),
  TilingWindow(TilingWindowRef),
  NonTilingWindow(NonTilingWindowRef),
}

impl ContainerRef {
  pub fn as_common(&self) -> &dyn CommonContainer {
    match self {
      ContainerRef::RootContainer(c) => c,
      ContainerRef::Monitor(c) => c,
      // ContainerRef::Workspace(c) => c,
      // ContainerRef::SplitContainer(c) => c,
      // ContainerRef::Window(c) => c,
      _ => todo!(),
    }
  }

  pub fn as_tiling(&self) -> &dyn TilingContainer {
    match self {
      ContainerRef::RootContainer(c) => c,
      ContainerRef::Monitor(c) => c,
      // ContainerRef::Workspace(c) => c,
      // ContainerRef::SplitContainer(c) => c,
      // ContainerRef::Window(c) => c,
      _ => todo!(),
    }
  }
}

#[derive(Clone, Debug)]
pub enum TilingContainerRef {
  RootContainer(RootContainerRef),
  Monitor(MonitorRef),
  Workspace(WorkspaceRef),
  SplitContainer(SplitContainerRef),
  TilingWindow(TilingWindowRef),
}
