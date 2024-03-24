use anyhow::anyhow;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
  common::platform::{NativeMonitor, NativeWindow, Platform},
  containers::{traits::TilingBehavior, Container, RootContainer},
  monitors::Monitor,
  user_config::{BindingModeConfig, ParsedConfig, UserConfig},
  windows::TilingWindow,
  wm_event::WmEvent,
  workspaces::Workspace,
};

pub struct WmState {
  /// Root node of the container tree. Monitors are the children of the
  /// root node, followed by workspaces, then split containers/windows.
  root_container: RootContainer,

  /// Containers (and their descendants) that have a pending redraw.
  containers_to_redraw: Vec<Container>,

  /// Whether native focus needs to be reassigned to the WM's focused
  /// container.
  has_pending_focus_sync: bool,

  /// Currently enabled binding modes.
  binding_modes: Vec<BindingModeConfig>,

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
    let native_monitors = Platform::monitors()?;

    for native_monitor in native_monitors {
      let monitor = Monitor::new(native_monitor);

      let workspace_config = config
        .workspace_config_for_monitor(&monitor, &self.workspaces())
        .ok_or(anyhow!("No workspace config found for monitor."))?;

      let workspace = Workspace::new(
        workspace_config,
        config.value.gaps.outer_gap.clone(),
      );

      monitor.insert_child(0, workspace.into());
      self.root_container.insert_child(0, monitor.into());
    }

    for native_window in Platform::manageable_windows()? {
      let nearest_monitor = Platform::nearest_monitor(&native_window)
        .and_then(|native| self.monitor_from_native(&native))
        .or(self.monitors().first().cloned());

      if let Some(monitor) = nearest_monitor {
        // TODO: This should actually add to the monitor's displayed workspace.
        let window = TilingWindow::new(native_window);
        monitor.insert_child(0, window.into());
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
      .map(|c| c.as_monitor().unwrap())
      .collect()
  }

  pub fn workspaces(&self) -> Vec<Workspace> {
    self
      .monitors()
      .iter()
      .flat_map(|c| c.children())
      .map(|c| c.as_workspace().unwrap())
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
    native_window: NativeWindow,
  ) -> Option<Container> {
    todo!()
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
