use std::sync::Arc;
use uuid::Uuid;

use crate::{
  common::FocusMode,
  containers::{Container, ContainerType, RootContainer},
};

pub struct WmState {
  /// The root node of the container tree. Monitors are the children of the
  /// root node, followed by workspaces, then split containers/windows.
  root_container: RootContainer,

  /// Containers (and their descendants) to redraw on the next invocation
  /// of `RedrawContainersCommand`.
  containers_to_redraw: Vec<Arc<dyn Container>>,

  /// Whether native focus needs to be reassigned to the WM's focused
  /// container.
  has_pending_focus_sync: bool,

  /// Name of the currently active binding mode (if one is active).
  active_binding_mode: Option<String>,
}

impl WmState {
  pub fn new() -> Self {
    Self {
      root_container: RootContainer::new(),
      containers_to_redraw: Vec::new(),
      has_pending_focus_sync: false,
      active_binding_mode: None,
    }
  }

  /// The currently focused container. This can either be a `Window` or a
  /// `Workspace` without any descendant windows.
  pub fn focused_container(&self) -> Arc<dyn Container> {
    self
      .root_container
      .last_focused_descendant()
      .unwrap()
      .clone()
  }

  /// Whether a tiling or floating container is currently focused.
  pub fn focus_mode(&self) -> FocusMode {
    match self.focused_container().r#type() {
      ContainerType::FloatingWindow => FocusMode::Floating,
      _ => FocusMode::Tiling,
    }
  }

  pub fn get_container_by_id(
    &self,
    id: Uuid,
  ) -> Option<Arc<dyn Container>> {
    self
      .root_container
      .self_and_descendants()
      .into_iter()
      .find(|container| container.id() == id)
  }
}
