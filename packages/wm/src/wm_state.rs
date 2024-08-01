use std::{collections::HashSet, time::Instant};

use anyhow::Context;
use tokio::sync::mpsc::{self};
use tracing::warn;
use uuid::Uuid;

use crate::{
  cleanup::run_cleanup,
  common::{
    commands::platform_sync,
    platform::{NativeMonitor, NativeWindow, Platform},
    Direction, Point,
  },
  containers::{
    commands::set_focused_descendant,
    traits::{CommonGetters, PositionGetters},
    Container, RootContainer, WindowContainer,
  },
  monitors::{commands::add_monitor, Monitor},
  user_config::{BindingModeConfig, UserConfig},
  windows::{commands::manage_window, traits::WindowGetters, WindowState},
  wm_event::WmEvent,
  workspaces::{Workspace, WorkspaceTarget},
};

pub struct WmState {
  /// Root node of the container tree. Monitors are the children of the
  /// root node, followed by workspaces, then split containers/windows.
  pub root_container: RootContainer,

  pub pending_sync: PendingSync,

  /// Name of the most recently focused workspace.
  ///
  /// Used for the `general.toggle_workspace_on_refocus` option on
  /// workspace focus.
  pub recent_workspace_name: Option<String>,

  /// Container that most recently had focus synced.
  ///
  /// Used for updating window effects on focus change.
  pub recent_focused_container: Option<Container>,

  /// Time since a previously focused window was unmanaged or minimized.
  ///
  /// Used to decide whether to override incoming focus events.
  pub unmanaged_or_minimized_timestamp: Option<Instant>,

  /// Configs of currently enabled binding modes.
  pub binding_modes: Vec<BindingModeConfig>,

  /// Windows that the WM should ignore. Windows can be added via the
  /// `ignore` command.
  pub ignored_windows: Vec<NativeWindow>,

  /// Whether the initial state has been populated.
  has_initialized: bool,

  /// Sender for emitting WM-related events.
  event_tx: mpsc::UnboundedSender<WmEvent>,

  /// Sender for gracefully shutting down the WM.
  exit_tx: mpsc::UnboundedSender<()>,
}

pub struct PendingSync {
  /// Containers (and their descendants) that have a pending redraw.
  pub containers_to_redraw: Vec<Container>,

  /// Whether native focus should be reassigned to the WM's focused
  /// container.
  pub focus_change: bool,

  /// Whether window effects should be reset.
  pub reset_window_effects: bool,

  /// Container to center the cursor on.
  pub cursor_container: Option<Container>,
}

impl WmState {
  pub fn new(
    event_tx: mpsc::UnboundedSender<WmEvent>,
    exit_tx: mpsc::UnboundedSender<()>,
  ) -> Self {
    Self {
      root_container: RootContainer::new(),
      pending_sync: PendingSync {
        containers_to_redraw: Vec::new(),
        focus_change: false,
        reset_window_effects: false,
        cursor_container: None,
      },
      recent_focused_container: None,
      recent_workspace_name: None,
      unmanaged_or_minimized_timestamp: None,
      binding_modes: Vec::new(),
      ignored_windows: Vec::new(),
      has_initialized: false,
      event_tx,
      exit_tx,
    }
  }

  /// Populates the initial WM state by creating containers for all
  /// existing windows and monitors.
  pub fn populate(
    &mut self,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    // Get the originally focused window when the WM was started.
    let foreground_window = Platform::foreground_window();

    // Create a monitor, and consequently a workspace, for each detected
    // native monitor.
    for native_monitor in Platform::sorted_monitors()? {
      add_monitor(native_monitor, self, config)?;
    }

    for native_window in Platform::manageable_windows()? {
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
      .or(self.windows().pop().map(Into::into))
      .or(self.workspaces().pop().map(Into::into))
      .context("Failed to get container to focus.")?;

    set_focused_descendant(container_to_focus, None);

    self.pending_sync.focus_change = true;
    self.pending_sync.reset_window_effects = true;
    platform_sync(self, config)?;

    self.has_initialized = true;
    Ok(())
  }

  pub fn monitors(&self) -> Vec<Monitor> {
    self
      .root_container
      .children()
      .iter()
      .filter_map(|container| container.as_monitor().cloned())
      .collect()
  }

  pub fn workspaces(&self) -> Vec<Workspace> {
    self
      .monitors()
      .iter()
      .flat_map(|monitor| monitor.workspaces())
      .collect()
  }

  /// Gets workspaces sorted by their position in the user config.
  pub fn sorted_workspaces(&self, config: &UserConfig) -> Vec<Workspace> {
    let mut workspaces = self.workspaces();
    config.sort_workspaces(&mut workspaces);
    workspaces
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
    self
      .monitor_from_native(&Platform::nearest_monitor(&native_window))
      .or(self.monitors().first().cloned())
  }

  /// Gets monitor that corresponds to the given `NativeMonitor`.
  pub fn monitor_from_native(
    &self,
    native_monitor: &NativeMonitor,
  ) -> Option<Monitor> {
    self
      .monitors()
      .into_iter()
      .find(|monitor| monitor.native() == *native_monitor)
  }

  /// Gets the closest monitor in a given direction.
  ///
  /// Uses i3wm's algorithm for finding best guess.
  pub fn monitor_in_direction(
    &self,
    origin_monitor: &Monitor,
    direction: &Direction,
  ) -> anyhow::Result<Option<Monitor>> {
    let origin_rect = origin_monitor.native().rect()?.clone();

    // Create a tuple of monitors and their rect.
    let monitors_with_rect = self
      .monitors()
      .into_iter()
      .map(|monitor| {
        let rect = monitor.native().rect()?.clone();
        anyhow::Ok((monitor, rect))
      })
      .try_collect::<Vec<_>>()?;

    let closest_monitor = monitors_with_rect
      .into_iter()
      .filter(|(_, rect)| match direction {
        Direction::Right => {
          rect.x() > origin_rect.x() && rect.has_overlap_y(&origin_rect)
        }
        Direction::Left => {
          rect.x() < origin_rect.x() && rect.has_overlap_y(&origin_rect)
        }
        Direction::Down => {
          rect.y() > origin_rect.y() && rect.has_overlap_x(&origin_rect)
        }
        Direction::Up => {
          rect.y() < origin_rect.y() && rect.has_overlap_x(&origin_rect)
        }
      })
      .min_by(|(_, rect_a), (_, rect_b)| match direction {
        Direction::Right => rect_a.x().cmp(&rect_b.x()),
        Direction::Left => rect_b.x().cmp(&rect_a.x()),
        Direction::Down => rect_a.y().cmp(&rect_b.y()),
        Direction::Up => rect_b.y().cmp(&rect_a.y()),
      })
      .map(|(monitor, _)| monitor);

    Ok(closest_monitor)
  }

  /// Gets window that corresponds to the given `NativeWindow`.
  pub fn window_from_native(
    &self,
    native_window: &NativeWindow,
  ) -> Option<WindowContainer> {
    self
      .windows()
      .into_iter()
      .find(|window| &*window.native() == native_window)
  }

  pub fn workspace_by_name(
    &self,
    workspace_name: &str,
  ) -> Option<Workspace> {
    self
      .workspaces()
      .into_iter()
      .find(|workspace| workspace.config().name == workspace_name)
  }

  /// Gets a workspace and its name by the given target.
  ///
  /// Returns a tuple of the workspace name and the `Workspace` instance
  /// if active.
  pub fn workspace_by_target(
    &self,
    origin_workspace: &Workspace,
    target: WorkspaceTarget,
    config: &UserConfig,
  ) -> anyhow::Result<(Option<String>, Option<Workspace>)> {
    let (name, workspace) = match target {
      WorkspaceTarget::Name(name) => {
        match origin_workspace.config().name == name {
          false => (Some(name.clone()), self.workspace_by_name(&name)),
          // Toggle the workspace if it's already focused.
          true if config.value.general.toggle_workspace_on_refocus => (
            self.recent_workspace_name.clone(),
            self
              .recent_workspace_name
              .as_ref()
              .and_then(|name| self.workspace_by_name(name)),
          ),
          true => (None, None),
        }
      }
      WorkspaceTarget::Recent => (
        self.recent_workspace_name.clone(),
        self
          .recent_workspace_name
          .as_ref()
          .and_then(|name| self.workspace_by_name(&name)),
      ),
      WorkspaceTarget::Next => {
        let workspaces = self.sorted_workspaces(config);
        let origin_index = workspaces
          .iter()
          .position(|workspace| workspace.id() == origin_workspace.id())
          .context("Failed to get index of given workspace.")?;

        let next_workspace = workspaces
          .get(origin_index + 1)
          .or_else(|| workspaces.first());

        (
          next_workspace.map(|workspace| workspace.config().name),
          next_workspace.cloned(),
        )
      }
      WorkspaceTarget::Previous => {
        let workspaces = self.sorted_workspaces(config);
        let origin_index = workspaces
          .iter()
          .position(|workspace| workspace.id() == origin_workspace.id())
          .context("Failed to get index of given workspace.")?;

        let prev_workspace = workspaces.get(
          origin_index.checked_sub(1).unwrap_or(workspaces.len() - 1),
        );

        (
          prev_workspace.map(|workspace| workspace.config().name),
          prev_workspace.cloned(),
        )
      }
      WorkspaceTarget::Direction(direction) => {
        let origin_monitor =
          origin_workspace.monitor().context("No focused monitor.")?;

        let target_workspace = self
          .monitor_in_direction(&origin_monitor, &direction)?
          .and_then(|monitor| monitor.displayed_workspace());

        (
          target_workspace
            .as_ref()
            .map(|workspace| workspace.config().name),
          target_workspace,
        )
      }
    };

    Ok((name, workspace))
  }

  /// Gets windows that should be redrawn.
  ///
  /// When redrawing after a command that changes a window's type (e.g.
  /// tiling -> floating), the original detached window might still be
  /// queued for a redraw and should be filtered out.
  pub fn windows_to_redraw(&self) -> Vec<WindowContainer> {
    let mut unique_ids = HashSet::new();

    self
      .pending_sync
      .containers_to_redraw
      .iter()
      .flat_map(|container| container.self_and_descendants())
      .filter(|container| !container.is_detached())
      .filter_map(|container| container.try_into().ok())
      .filter(|window: &WindowContainer| unique_ids.insert(window.id()))
      .collect()
  }

  /// Gets the currently focused container. This can either be a window or
  /// a workspace without any descendant windows.
  pub fn focused_container(&self) -> Option<Container> {
    self.root_container.descendant_focus_order().next()
  }

  /// Emits a WM event through an MSPC channel.
  ///
  /// Does not emit events while the WM is populating initial state. This
  /// is to prevent events (e.g. workspace activation events) from being
  /// emitted via IPC server before the initial state is prepared.
  pub fn emit_event(&self, event: WmEvent) {
    if self.has_initialized {
      if let Err(err) = self.event_tx.send(event) {
        warn!("Failed to send event: {}", err);
      }
    }
  }

  /// Starts graceful shutdown via an MSPC channel.
  pub fn emit_exit(&self) {
    self.exit_tx.send(()).unwrap()
  }

  pub fn container_by_id(&self, id: Uuid) -> Option<Container> {
    self
      .root_container
      .self_and_descendants()
      .into_iter()
      .find(|container| container.id() == id)
  }

  /// Gets container to focus after the given window is unmanaged,
  /// minimized, or moved to another workspace.
  pub fn focus_target_after_removal(
    &self,
    removed_window: &WindowContainer,
  ) -> Option<Container> {
    // If the removed window is not focused, no need to change focus.
    if self.focused_container() != Some(removed_window.clone().into()) {
      return None;
    }

    // Get descendant focus order excluding the removed container.
    let workspace = removed_window.workspace()?;
    let descendant_focus_order = workspace
      .descendant_focus_order()
      .filter(|descendant| descendant.id() != removed_window.id())
      .collect::<Vec<_>>();

    // Get focus target that matches the removed window type. This applies
    // for windows that aren't in a minimized state.
    let focus_target_of_type = descendant_focus_order
      .iter()
      .filter_map(|descendant| descendant.as_window_container().ok())
      .find(|descendant| {
        match (descendant.state(), removed_window.state()) {
          (WindowState::Tiling, WindowState::Tiling) => true,
          (WindowState::Floating(_), WindowState::Floating(_)) => true,
          (WindowState::Fullscreen(_), WindowState::Fullscreen(_)) => true,
          _ => false,
        }
      })
      .map(Into::into);

    if focus_target_of_type.is_some() {
      return focus_target_of_type;
    }

    let non_minimized_focus_target = descendant_focus_order
      .iter()
      .filter_map(|descendant| descendant.as_window_container().ok())
      .find(|descendant| descendant.state() != WindowState::Minimized)
      .map(Into::into);

    non_minimized_focus_target
      .or(descendant_focus_order.first().cloned())
      .or(Some(workspace.into()))
  }

  /// Returns all window under the mouse position
  pub fn window_containers_at_position(
    &self,
    position: &Point,
  ) -> Vec<WindowContainer> {
    self
      .root_container
      .descendants()
      .filter_map(|container| match container {
        Container::TilingWindow(tiling) => {
          Some(WindowContainer::TilingWindow(tiling))
        }
        Container::NonTilingWindow(non_tiling) => {
          Some(WindowContainer::NonTilingWindow(non_tiling))
        }
        _ => None,
      })
      .filter(|c| {
        let frame = c.to_rect();
        frame.unwrap().contains_point(&position)
      })
      .collect()
  }
}

impl Drop for WmState {
  fn drop(&mut self) {
    let managed_windows = self
      .windows()
      .into_iter()
      .map(|window| window.native().clone())
      .collect::<Vec<_>>();

    run_cleanup(managed_windows);
  }
}
