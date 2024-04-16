use anyhow::Context;
use tokio::sync::mpsc::{self};
use tracing::warn;
use uuid::Uuid;

use crate::{
  common::{
    commands::sync_native_focus,
    platform::{NativeMonitor, NativeWindow, Platform},
    FocusMode,
  },
  containers::{
    commands::{
      handle_set_active_window_border, redraw, set_focused_descendant,
    },
    traits::CommonGetters,
    Container, RootContainer, WindowContainer,
  },
  monitors::{commands::add_monitor, Monitor},
  user_config::UserConfig,
  windows::{commands::manage_window, traits::WindowGetters},
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

  pub active_border_window: Option<NativeWindow>,

  /// Names of any currently enabled binding modes.
  pub binding_modes: Vec<String>,

  /// Any appbar windows (application desktop toolbars) that have been
  /// detected. Positioning changes of appbars can affect the working area
  /// of the parent monitor, and requires windows to be redrawn.
  pub app_bar_windows: Vec<NativeWindow>,

  /// Sender for emitting WM-related events.
  event_tx: mpsc::UnboundedSender<WmEvent>,
}

impl WmState {
  pub fn new(event_tx: mpsc::UnboundedSender<WmEvent>) -> Self {
    Self {
      root_container: RootContainer::new(),
      containers_to_redraw: Vec::new(),
      has_pending_focus_sync: false,
      active_border_window: None,
      binding_modes: Vec::new(),
      app_bar_windows: Vec::new(),
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
      // Register appbar windows.
      if native_window.is_app_bar() {
        self.app_bar_windows.push(native_window.clone());
      }

      let nearest_workspace = self
        .nearest_monitor(&native_window)
        .and_then(|m| m.displayed_workspace());

      if let Some(workspace) = nearest_workspace {
        manage_window(
          native_window,
          Some(workspace.into()),
          self,
          config,
        )?;
      }
    }

    let container_to_focus = self
      .window_from_native(&foreground_window)
      .map(|c| c.as_container())
      .or(self.windows().pop().map(|c| c.into()))
      .or(self.workspaces().pop().map(|c| c.into()))
      .context("Failed to get container to focus.")?;

    set_focused_descendant(container_to_focus, None, self);
    self.has_pending_focus_sync = true;

    redraw(self, config)?;
    sync_native_focus(self)?;

    Ok(())
  }

  pub fn monitors(&self) -> Vec<Monitor> {
    self
      .root_container
      .children()
      .iter()
      .filter_map(|c| c.as_monitor().cloned())
      .collect()
  }

  pub fn workspaces(&self) -> Vec<Workspace> {
    self
      .monitors()
      .iter()
      .flat_map(|c| c.children())
      .filter_map(|c| c.as_workspace().cloned())
      .collect()
  }

  pub fn windows(&self) -> Vec<WindowContainer> {
    self
      .root_container
      .descendants()
      .filter_map(|container| container.try_into().ok())
      .collect()
  }

  /// Gets the monitor that encompasses the largest portion of a given
  /// window.
  ///
  /// Defaults to the first monitor if the nearest monitor is invalid.
  pub fn nearest_monitor(
    &self,
    native_window: &NativeWindow,
  ) -> Option<Monitor> {
    Platform::nearest_monitor(&native_window)
      .ok()
      .and_then(|native| self.monitor_from_native(&native))
      .or(self.monitors().first().cloned())
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
  ) -> Option<WindowContainer> {
    self
      .windows()
      .iter()
      .find(|w| w.native() == *native_window)
      .cloned()
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

  pub fn add_container_to_redraw(&mut self, container: Container) {
    self.containers_to_redraw.push(container);
  }

  /// Removes all containers from the redraw queue.
  pub fn clear_containers_to_redraw(&mut self) {
    self.containers_to_redraw.clear();
  }

  /// Gets the currently focused container. This can either be a window or
  /// a workspace without any descendant windows.
  pub fn focused_container(&self) -> Option<Container> {
    self.root_container.last_focused_descendant()
  }

  /// Whether a tiling or floating container is currently focused.
  pub fn focus_mode(&self) -> Option<FocusMode> {
    self.focused_container().map(|c| match c {
      Container::NonTilingWindow(_) => FocusMode::Floating,
      _ => FocusMode::Tiling,
    })
  }

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
