use crate::{
  monitors::MonitorRef, windows::WindowRef, workspaces::WorkspaceRef,
};

use super::{CommonContainer, RootContainerRef, SplitContainerRef};

/// A reference to a `Container`.
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
  Window(WindowRef),
}

impl ContainerRef {
  pub fn common(&self) -> &dyn CommonContainer {
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
