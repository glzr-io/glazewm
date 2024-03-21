use std::{
  borrow::BorrowMut,
  cell::RefCell,
  rc::{Rc, Weak},
  sync::Arc,
};

use anyhow::Result;
use tokio::sync::{mpsc::UnboundedSender, Mutex};
use tracing::warn;
use uuid::Uuid;

use crate::{
  common::{
    platform::{NativeMonitor, Platform},
    FocusMode,
  },
  containers::{
    CommonContainer, ContainerRef, ContainerType, RootContainer,
    RootContainerRef,
  },
  monitors::{Monitor, MonitorRef},
  user_config::{BindingModeConfig, UserConfig},
  wm_event::WmEvent,
};

pub struct WmState {
  /// Root node of the container tree. Monitors are the children of the
  /// root node, followed by workspaces, then split containers/windows.
  root_container: RootContainerRef,

  /// Containers (and their descendants) that have a pending redraw.
  containers_to_redraw: Vec<Weak<ContainerRef>>,

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
      root_container: RootContainerRef::new(),
      containers_to_redraw: Vec::new(),
      has_pending_focus_sync: false,
      binding_modes: Vec::new(),
      config,
      config_changes_tx,
      event_tx,
    }
  }

  pub fn populate(&mut self) -> anyhow::Result<()> {
    let foreground_window = Platform::foreground_window();
    let native_monitors = Platform::monitors()?;

    for native_monitor in native_monitors {
      let monitor = MonitorRef::new(native_monitor);

      self
        .root_container
        .insert_child(0, ContainerRef::Monitor(monitor));
    }

    for native_window in Platform::manageable_windows()? {
      let window = ContainerRef::Window(native_window);
      self.root_container.insert_child(0, window);
    }

    Ok(())
  }

  pub fn nearest_monitor(&self) -> Option<MonitorRef> {
    self
      .root_container
      .self_and_descendants()
      .into_iter()
      .filter_map(|container| match container {
        ContainerRef::Monitor(monitor) => Some(monitor.clone()),
        _ => None,
      })
      .next()
  }

  pub fn add_monitor(&mut self) {
    let monitor = MonitorRef::new(NativeMonitor::new(
      windows::Win32::Graphics::Gdi::HMONITOR(1),
      String::from("aaa"),
      0,
      0,
      0,
      0,
    ));

    self
      .root_container
      .insert_child(0, ContainerRef::Monitor(monitor))
  }

  pub fn emit_event(&self, event: WmEvent) -> Result<()> {
    if let Err(err) = self.event_tx.send(event) {
      warn!("Failed to send event: {}", err);
    }

    Ok(())
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
