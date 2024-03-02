use std::{
  borrow::BorrowMut,
  cell::RefCell,
  rc::{Rc, Weak},
  sync::Arc,
};

use anyhow::Result;
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use uuid::Uuid;

use crate::{
  common::FocusMode,
  containers::{Container, ContainerType, RootContainer},
  monitors::Monitor,
  user_config::{BindingModeConfig, UserConfig},
  wm_event::WmEvent,
};

pub struct WmState {
  /// Root node of the container tree. Monitors are the children of the
  /// root node, followed by workspaces, then split containers/windows.
  root_container: RootContainer,

  /// Containers (and their descendants) that have a pending redraw.
  containers_to_redraw: Vec<Weak<Container>>,

  /// Whether native focus needs to be reassigned to the WM's focused
  /// container.
  has_pending_focus_sync: bool,

  /// Currently enabled binding modes.
  binding_modes: Vec<BindingModeConfig>,

  /// Parsed user config.
  config: Arc<Mutex<UserConfig>>,

  config_changes_tx: UnboundedSender<UserConfig>,

  event_tx: UnboundedSender<WmEvent>,
}

impl WmState {
  pub fn new(
    config: Arc<Mutex<UserConfig>>,
    config_changes_tx: UnboundedSender<UserConfig>,
    event_tx: UnboundedSender<WmEvent>,
  ) -> Self {
    Self {
      root_container: RootContainer::new(),
      containers_to_redraw: Vec::new(),
      has_pending_focus_sync: false,
      binding_modes: Vec::new(),
      config,
      config_changes_tx,
      event_tx,
    }
  }

  pub fn add_monitor(&mut self) {
    // self.root_container.add_monitor(monitor_id);
    let monitor = Monitor::new(String::from("aaa"), 0, 0, 0, 0);
    self
      .root_container
      .inner
      .children
      .borrow_mut()
      .push_front(Rc::new(RefCell::new(Container::Monitor(monitor))));
  }

  pub fn emit_event(&self, event: WmEvent) -> Result<()> {
    self.event_tx.send(event)
  }

  // Get the currently focused container. This can either be a `Window` or
  // a `Workspace` without any descendant windows.
  // pub fn focused_container(&self) -> Arc<Container> {
  //   self
  //     .root_container
  //     .last_focused_descendant()
  //     .unwrap()
  //     .clone()
  // }

  // /// Whether a tiling or floating container is currently focused.
  // pub fn focus_mode(&self) -> FocusMode {
  //   match self.focused_container().r#type() {
  //     ContainerType::FloatingWindow => FocusMode::Floating,
  //     _ => FocusMode::Tiling,
  //   }
  // }

  // pub fn container_by_id(&self, id: Uuid) -> Option<Arc<Container>> {
  //   self
  //     .root_container
  //     .self_and_descendants()
  //     .into_iter()
  //     .find(|container| container.id() == id)
  // }
}
