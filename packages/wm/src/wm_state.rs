use tokio::sync::mpsc::UnboundedSender;
use tracing::warn;
use uuid::Uuid;

use crate::{
  common::platform::{NativeMonitor, NativeWindow, Platform},
  containers::{
    commands::attach_container, traits::CommonBehavior, Container,
    RootContainer, WindowContainer,
  },
  monitors::{commands::add_monitor, Monitor},
  user_config::{BindingModeConfig, UserConfig},
  windows::TilingWindow,
  wm_event::WmEvent,
  workspaces::Workspace,
};

pub struct WmState {
  /// Root node of the container tree. Monitors are the children of the
  /// root node, followed by workspaces, then split containers/windows.
  pub root_container: RootContainer,

  /// Containers (and their descendants) that have a pending redraw.
  containers_to_redraw: Vec<Container>,

  /// Whether native focus needs to be reassigned to the WM's focused
  /// container.
  pub has_pending_focus_sync: bool,

  /// Currently enabled binding modes.
  pub binding_modes: Vec<BindingModeConfig>,

  event_tx: UnboundedSender<WmEvent>,
}

impl WmState {
  pub fn new(event_tx: UnboundedSender<WmEvent>) -> Self {
    Self {
      root_container: RootContainer::new(),
      containers_to_redraw: Vec::new(),
      has_pending_focus_sync: false,
      binding_modes: Vec::new(),
      event_tx,
    }
  }

  /// Populates the initial WM state by creating containers for all
  /// existing windows and monitors.
  pub fn populate(&mut self, config: &UserConfig) -> anyhow::Result<()> {
    // Get the originally focused window when the WM was started.
    let foreground_window = Platform::foreground_window();

    // Create a monitor, and consequently a workspace, for each detected
    // native monitor.
    for native_monitor in Platform::monitors()? {
      add_monitor(native_monitor, self, config)?;
    }

    for native_window in Platform::manageable_windows()? {
      let nearest_monitor = Platform::nearest_monitor(&native_window)
        .and_then(|native| self.monitor_from_native(&native))
        .or(self.monitors().first().cloned());

      if let Some(monitor) = nearest_monitor {
        // TODO: This should actually add to the monitor's displayed workspace.
        let window = TilingWindow::new(native_window);
        attach_container(window.into(), &monitor.into(), 0)?;
      }
    }

    self.has_pending_focus_sync = true;
    Ok(())
  }

  pub fn monitors(&self) -> Vec<Monitor> {
    self
      .root_container
      .children()
      .iter()
      .map(|c| c.as_monitor().cloned().unwrap())
      .collect()
  }

  pub fn workspaces(&self) -> Vec<Workspace> {
    self
      .monitors()
      .iter()
      .flat_map(|c| c.children())
      .map(|c| c.as_workspace().cloned().unwrap())
      .collect()
  }

  pub fn monitor_from_native(
    &self,
    native_monitor: &NativeMonitor,
  ) -> Option<Monitor> {
    self
      .monitors()
      .iter()
      .find(|&m| m.native() == *native_monitor)
      .cloned()
  }

  pub fn window_from_native(
    &self,
    native_window: &NativeWindow,
  ) -> Option<&Container> {
    todo!()
  }

  /// Gets windows that should be redrawn.
  ///
  /// When redrawing after a command that changes a window's type (eg.
  /// tiling -> floating), the original detached window might still be
  /// queued for a redraw and should be ignored.
  pub fn windows_to_redraw(&self) -> Vec<WindowContainer> {
    self
      .containers_to_redraw
      .iter()
      .flat_map(|container| container.self_and_descendants())
      .filter(|container| !container.is_detached())
      .filter_map(|container| container.try_into().ok())
      // .unique()
      .collect()
  }

  /// Removes all containers from the redraw queue.
  pub fn clear_containers_to_redraw(&mut self) {
    self.containers_to_redraw.clear();
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

  pub fn emit_event(&self, event: WmEvent) {
    if let Err(err) = self.event_tx.send(event) {
      warn!("Failed to send event: {}", err);
    }
  }

  pub fn container_by_id(&self, id: Uuid) -> Option<Container> {
    self
      .root_container
      .self_and_descendants()
      .into_iter()
      .find(|container| container.id() == id)
  }
}
