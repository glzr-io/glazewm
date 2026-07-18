use std::{
  collections::HashSet,
  time::{Duration, Instant},
};

use anyhow::Context;
use tokio::sync::mpsc::{self};
use tracing::warn;
use uuid::Uuid;
use wm_common::{BindingModeConfig, HideCorner, WindowState, WmEvent};
use wm_platform::{
  Direction, Dispatcher, Display, NativeWindow, Point, Rect, WindowId,
};
#[cfg(target_os = "windows")]
use wm_platform::{NativeWindowWindowsExt, OpacityValue};

use crate::{
  commands::{
    container::set_focused_descendant,
    general::platform_sync,
    monitor::{add_monitor, move_bounded_workspaces_to_new_monitor},
    window::{manage_window, unmanage_window},
    workspace::activate_workspace,
  },
  models::{
    Container, Monitor, NativeMonitorProperties, RootContainer,
    WindowContainer, Workspace, WorkspaceTarget,
  },
  pending_sync::PendingSync,
  traits::{CommonGetters, PositionGetters, WindowGetters},
  user_config::UserConfig,
};

/// A window released to the OS (`multi_monitor_workspaces: false`) that
/// should be managed again once it is moved onto a monitor with
/// workspaces.
pub(crate) struct PendingRemanage {
  /// The released native window.
  pub window: NativeWindow,

  /// Present while a WM-initiated snap to a workspaceless monitor is
  /// still settling. While set, programmatic moves must not trigger a
  /// re-manage (see `RemanageSnapGuard`).
  pub snap_guard: Option<RemanageSnapGuard>,
}

/// Guard on a released window's pending re-manage while a WM-initiated
/// snap is settling.
///
/// Repositions are issued asynchronously (`SWP_ASYNCWINDOWPOS`), so moves
/// queued *before* a window was unmanaged can land *after* it, and their
/// location changes are indistinguishable from genuine programmatic moves
/// (e.g. `Win+Shift+Arrow`). Re-managing on such an echo starts an
/// unmanage/re-manage feedback loop that bounces the window between
/// monitors (notably when the OS restores windows to a re-plugged
/// monitor).
#[derive(Clone, Debug)]
pub(crate) struct RemanageSnapGuard {
  /// ID of the monitor the window was snapped to.
  monitor_id: Uuid,

  /// When the guard expires. Safety valve for when the snap never
  /// produces an observable location change (e.g. rejected by the OS).
  expires_at: Instant,
}

impl RemanageSnapGuard {
  /// Duration after which a snap is assumed to have settled.
  const SETTLE_TIMEOUT: Duration = Duration::from_secs(1);

  fn new(monitor_id: Uuid) -> Self {
    Self {
      monitor_id,
      expires_at: Instant::now() + Self::SETTLE_TIMEOUT,
    }
  }

  /// Whether the snap has settled.
  ///
  /// Settles once the window is observed on the snapped-to monitor, if
  /// that monitor no longer exists (e.g. it was disconnected), or after
  /// the guard expires.
  fn is_settled(
    &self,
    nearest_monitor_id: Uuid,
    snap_monitor_exists: bool,
    now: Instant,
  ) -> bool {
    now >= self.expires_at
      || !snap_monitor_exists
      || nearest_monitor_id == self.monitor_id
  }
}

pub struct WmState {
  /// Root node of the container tree. Monitors are the children of the
  /// root node, followed by workspaces, then split containers/windows.
  pub root_container: RootContainer,

  pub dispatcher: Dispatcher,

  pub pending_sync: PendingSync,

  /// Name of the most recently focused workspace.
  ///
  /// Used for the `general.toggle_workspace_on_refocus` option on
  /// workspace focus.
  pub recent_workspace_name: Option<String>,

  /// The previously focused window that had focus effects applied.
  ///
  /// Used to efficiently update window effects by only removing focus
  /// effects from the previous window rather than all windows when focus
  /// changes.
  pub prev_effects_window: Option<WindowContainer>,

  /// Time since a previously focused window was unmanaged or minimized.
  ///
  /// Used to decide whether to override incoming focus events.
  pub unmanaged_or_minimized_timestamp: Option<Instant>,

  /// Configs of currently enabled binding modes.
  pub binding_modes: Vec<BindingModeConfig>,

  /// Windows that the WM should ignore. Windows can be added via the
  /// `ignore` command.
  pub ignored_windows: Vec<NativeWindow>,

  /// Windows `GlazeWM` released with `multi_monitor_workspaces: false` so they
  /// could stay on a display without workspaces. Attempt to `manage_window`
  /// again when the user finishes moving one onto the primary monitor.
  pub(crate) native_windows_pending_remanage: Vec<PendingRemanage>,

  /// Windows-only: `HWND`s with an active `EVENT_SYSTEM_MOVESIZE*` session so
  /// `EVENT_OBJECT_LOCATIONCHANGE` can be told apart from Win+Shift+Arrow
  /// moves (location-only, no interactive start/end).
  pub(crate) native_windows_in_interactive_move: HashSet<WindowId>,

  /// When the most recent display settings change was observed.
  ///
  /// While a change is still settling (see `is_display_change_settling`),
  /// the OS reshuffles windows across the new topology and the WM's own
  /// async repositions are still landing. Reactive unmanage/re-manage
  /// based on these transient moves must be suppressed, otherwise it
  /// starts a self-sustaining monitor-to-monitor flicker.
  last_display_change_at: Option<Instant>,

  /// Whether the WM is paused.
  pub is_paused: bool,

  /// Whether the OS focused window is the same as the WM focused window.
  pub is_focus_synced: bool,

  /// Whether the initial state has been populated.
  has_initialized: bool,

  /// Sender for emitting WM-related events.
  event_tx: mpsc::UnboundedSender<WmEvent>,

  /// Sender for gracefully shutting down the WM.
  exit_tx: mpsc::UnboundedSender<()>,
}

impl WmState {
  pub fn new(
    dispatcher: Dispatcher,
    event_tx: mpsc::UnboundedSender<WmEvent>,
    exit_tx: mpsc::UnboundedSender<()>,
  ) -> Self {
    Self {
      root_container: RootContainer::new(),
      dispatcher,
      pending_sync: PendingSync::default(),
      prev_effects_window: None,
      recent_workspace_name: None,
      unmanaged_or_minimized_timestamp: None,
      binding_modes: Vec::new(),
      ignored_windows: Vec::new(),
      native_windows_pending_remanage: Vec::new(),
      native_windows_in_interactive_move: HashSet::default(),
      last_display_change_at: None,
      is_paused: false,
      is_focus_synced: false,
      has_initialized: false,
      event_tx,
      exit_tx,
    }
  }

  /// Duration after a display settings change during which reactive
  /// unmanage/re-manage based on non-interactive window moves is
  /// suppressed. Refreshed on each display change event, so cascading
  /// changes (common while a monitor is being connected) extend it.
  const DISPLAY_CHANGE_SETTLE_TIMEOUT: Duration = Duration::from_secs(2);

  /// Records that a display settings change was just observed.
  pub(crate) fn note_display_change(&mut self) {
    self.last_display_change_at = Some(Instant::now());
  }

  /// Whether a display settings change is still settling.
  ///
  /// During this window the OS moves windows around the new monitor
  /// topology and the WM's own async repositions are still landing.
  /// Treating those transient moves as user gestures (i.e. unmanaging a
  /// window pushed onto a workspaceless monitor, or re-managing one pushed
  /// back) starts a monitor-to-monitor flicker loop, so callers use this
  /// to fall back to simply re-asserting each window's slot.
  pub(crate) fn is_display_change_settling(&self) -> bool {
    self.last_display_change_at.is_some_and(|at| {
      at.elapsed() < Self::DISPLAY_CHANGE_SETTLE_TIMEOUT
    })
  }

  /// Registers a native handle for possible re-management once it is moved
  /// back onto the primary monitor (`multi_monitor_workspaces: false`).
  ///
  /// Pass `snap_monitor_id` when the WM snapped the window to a
  /// workspaceless monitor as part of releasing it, so that late-landing
  /// echoes of the WM's own async repositions don't trigger a re-manage
  /// while the snap is settling.
  pub(crate) fn register_native_window_pending_remanage(
    &mut self,
    window: NativeWindow,
    snap_monitor_id: Option<Uuid>,
  ) {
    self
      .native_windows_pending_remanage
      .retain(|entry| entry.window.id() != window.id());

    self.native_windows_pending_remanage.push(PendingRemanage {
      window,
      snap_guard: snap_monitor_id.map(RemanageSnapGuard::new),
    });
  }

  /// If `native_window` is registered for re-management, drops that entry and
  /// returns `true`.
  pub(crate) fn take_native_window_pending_remanage(
    &mut self,
    native_window: &NativeWindow,
  ) -> bool {
    if let Some(pos) = self
      .native_windows_pending_remanage
      .iter()
      .position(|entry| entry.window == *native_window)
    {
      self.native_windows_pending_remanage.swap_remove(pos);
      true
    } else {
      false
    }
  }

  /// Whether a WM-initiated snap for the given released window is still
  /// settling.
  ///
  /// While settling, programmatic location changes must not trigger a
  /// re-manage (see `RemanageSnapGuard`). Clears the guard as a side
  /// effect once it settles.
  pub(crate) fn is_pending_remanage_snap_settling(
    &mut self,
    native_window: &NativeWindow,
    nearest_monitor_id: Uuid,
  ) -> bool {
    let Some(index) = self
      .native_windows_pending_remanage
      .iter()
      .position(|entry| entry.window == *native_window)
    else {
      return false;
    };

    let Some(guard) =
      self.native_windows_pending_remanage[index].snap_guard.clone()
    else {
      return false;
    };

    let snap_monitor_exists = self
      .monitors()
      .iter()
      .any(|monitor| monitor.id() == guard.monitor_id);

    if guard.is_settled(
      nearest_monitor_id,
      snap_monitor_exists,
      Instant::now(),
    ) {
      self.native_windows_pending_remanage[index].snap_guard = None;
      return false;
    }

    true
  }

  /// Populates the initial WM state by creating containers for all
  /// existing windows and monitors.
  pub fn populate(
    &mut self,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    // Get the originally focused window when the WM was started.
    let focused_window = self.dispatcher.focused_window().ok();

    // Create monitors for all detected native displays first, so that
    // `state.primary_monitor(config)` lookups during workspace assignment
    // can resolve the configured primary regardless of its position in the
    // sorted order.
    let mut new_monitors = Vec::new();
    for native_display in self.dispatcher.sorted_displays()? {
      if let Ok(native_properties) =
        NativeMonitorProperties::try_from(&native_display)
      {
        let monitor =
          add_monitor(native_display, native_properties, self)?;
        new_monitors.push(monitor);
      }
    }

    // Assign workspaces to each monitor in a second pass.
    for monitor in new_monitors {
      move_bounded_workspaces_to_new_monitor(&monitor, self, config)?;
    }

    // When `multi_monitor_workspaces` is disabled, minimized windows on
    // the primary monitor are deferred and spread across workspaces
    // afterwards (see `distribute_minimized_windows_across_workspaces`)
    // instead of being left minimized.
    let distribute_minimized =
      !config.value.general.multi_monitor_workspaces;
    let mut minimized_windows = Vec::new();

    // Manage windows in reverse z-order (bottom to top). This helps to
    // preserve the original stacking order.
    for native_window in
      self.dispatcher.visible_windows()?.into_iter().rev()
    {
      let nearest_workspace = self
        .nearest_monitor(&native_window)
        .and_then(|m| m.displayed_workspace());

      if let Some(workspace) = nearest_workspace {
        if distribute_minimized
          && native_window.is_minimized().unwrap_or(false)
        {
          minimized_windows.push(native_window);
        } else {
          manage_window(
            native_window,
            Some(workspace.into()),
            self,
            config,
          )?;
        }
      } else if !config.value.general.multi_monitor_workspaces {
        // The window is on a monitor without workspaces (i.e. a
        // non-primary monitor with `multi_monitor_workspaces: false`).
        // Leave it OS-managed so that workspace changes on the primary
        // monitor don't affect it, and re-manage it once it is moved
        // onto the primary monitor.
        self.register_native_window_pending_remanage(native_window, None);
      }
    }

    self.distribute_minimized_windows_across_workspaces(
      minimized_windows,
      config,
    )?;

    let container_to_focus = focused_window
      .and_then(|focused_window| {
        self.window_from_native(&focused_window).map(Into::into)
      })
      .or_else(|| self.windows().pop().map(Into::into))
      .or_else(|| self.workspaces().pop().map(Into::into))
      .context("Failed to get container to focus.")?;

    set_focused_descendant(&container_to_focus, None);
    self.is_focus_synced = true;

    self
      .pending_sync
      .queue_focus_change()
      .queue_all_effects_update();

    for workspace in self.workspaces() {
      self.pending_sync.queue_workspace_to_reorder(workspace);
    }

    platform_sync(self, config)?;
    self.has_initialized = true;

    Ok(())
  }

  /// Restores each minimized window and attaches it to its own workspace,
  /// one window per configured workspace in order. Any windows beyond the
  /// number of configured workspaces are piled into the last workspace.
  ///
  /// Used on startup (with `multi_monitor_workspaces` disabled) so that
  /// minimized windows on the primary monitor are spread across workspaces
  /// as tiling windows instead of being left minimized. Target workspaces
  /// are activated on demand.
  ///
  /// Best-effort: a window that fails to attach is skipped so the rest are
  /// still placed.
  fn distribute_minimized_windows_across_workspaces(
    &mut self,
    windows: Vec<NativeWindow>,
    config: &mut UserConfig,
  ) -> anyhow::Result<()> {
    if windows.is_empty() {
      return Ok(());
    }

    // Configured workspace names, in config order.
    let workspace_names = config
      .value
      .workspaces
      .iter()
      .map(|workspace_config| workspace_config.name.clone())
      .collect::<Vec<_>>();

    // No workspaces configured; nothing to distribute into.
    let Some(last_index) = workspace_names.len().checked_sub(1) else {
      return Ok(());
    };

    for (index, native_window) in windows.into_iter().enumerate() {
      // Overflow windows are piled into the last workspace.
      let target_name = &workspace_names[index.min(last_index)];

      // Resolve the target workspace, activating it if it isn't active.
      let workspace = match self.workspace_by_name(target_name) {
        Some(workspace) => workspace,
        None => {
          let primary_monitor = self.primary_monitor(config);

          activate_workspace(
            Some(target_name),
            primary_monitor,
            self,
            config,
          )?;

          match self.workspace_by_name(target_name) {
            Some(workspace) => workspace,
            None => {
              warn!(
                "Workspace '{target_name}' missing after activation; \
                 skipping minimized window."
              );
              continue;
            }
          }
        }
      };

      // Restore the window so it is managed as a tiling window rather than
      // staying minimized.
      #[cfg(target_os = "windows")]
      if let Err(err) = native_window.restore(None) {
        warn!(?err, "Failed to restore minimized window before managing.");
      }

      manage_window(native_window, Some(workspace.into()), self, config)?;
    }

    Ok(())
  }

  pub fn monitors(&self) -> Vec<Monitor> {
    self.root_container.monitors()
  }

  /// Gets the primary monitor based on the user config.
  ///
  /// If `general.primary_monitor_hardware_id` is set, returns the monitor whose
  /// hardware ID or display UUID matches (see config docs). Otherwise, falls back
  /// to the leftmost monitor (index 0).
  pub fn primary_monitor(
    &self,
    config: &UserConfig,
  ) -> Option<Monitor> {
    let monitors = self.monitors();

    if let Some(id) =
      config.value.general.primary_monitor_hardware_id.as_deref()
    {
      let matched = monitors
        .iter()
        .find(|m| Self::monitor_matches_primary_hardware_id(m, id))
        .cloned();

      if matched.is_some() {
        return matched;
      }
    }

    monitors.into_iter().next()
  }

  /// Whether this monitor matches `general.primary_monitor_hardware_id` in config.
  ///
  /// # Platform-specific
  ///
  /// - **Windows**: Compares against the EDID-derived hardware ID.
  /// - **macOS**: Compares against the CoreGraphics display UUID.
  #[cfg(target_os = "windows")]
  fn monitor_matches_primary_hardware_id(
    monitor: &Monitor,
    id: &str,
  ) -> bool {
    monitor
      .native_properties()
      .hardware_id
      .as_deref()
      == Some(id)
  }

  #[cfg(target_os = "macos")]
  fn monitor_matches_primary_hardware_id(
    monitor: &Monitor,
    id: &str,
  ) -> bool {
    monitor.native_properties().device_uuid == id
  }

  #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
  fn monitor_matches_primary_hardware_id(
    monitor: &Monitor,
    id: &str,
  ) -> bool {
    let _ = (monitor, id);
    false
  }

  pub fn workspaces(&self) -> Vec<Workspace> {
    self
      .monitors()
      .iter()
      .flat_map(Monitor::workspaces)
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
      .monitor_from_native(
        &self.dispatcher.nearest_display(native_window).ok()?,
      )
      .or(self.monitors().first().cloned())
  }

  /// Gets monitor that corresponds to the given `Display`.
  pub fn monitor_from_native(
    &self,
    native_display: &Display,
  ) -> Option<Monitor> {
    self
      .monitors()
      .into_iter()
      .find(|monitor| monitor.native() == *native_display)
  }

  /// Gets the closest monitor in a given direction.
  ///
  /// Uses i3wm's algorithm for finding best guess.
  pub fn monitor_in_direction(
    &self,
    origin_monitor: &Monitor,
    direction: &Direction,
  ) -> anyhow::Result<Option<Monitor>> {
    let origin_rect = origin_monitor.native_properties().bounds;

    // Create a tuple of monitors and their rect.
    let monitors_with_rect = self
      .monitors()
      .into_iter()
      .map(|monitor| {
        let rect = monitor.native_properties().bounds;
        anyhow::Ok((monitor, rect))
      })
      .try_collect::<Vec<_>>()?;

    let closest_monitor = monitors_with_rect
      .into_iter()
      .filter(|(_, rect)| match direction {
        Direction::Right => {
          rect.x() > origin_rect.x() && rect.y_overlap(&origin_rect) > 0
        }
        Direction::Left => {
          rect.x() < origin_rect.x() && rect.y_overlap(&origin_rect) > 0
        }
        Direction::Down => {
          rect.y() > origin_rect.y() && rect.x_overlap(&origin_rect) > 0
        }
        Direction::Up => {
          rect.y() < origin_rect.y() && rect.x_overlap(&origin_rect) > 0
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

  /// Determines the preferred hide corner for each monitor. Used for
  /// [`HideMethod::PlaceInCorner`].
  ///
  /// The corner is chosen by simulating a 400x400 window frame in the
  /// bottom-left and bottom-right of the monitor's working area, then
  /// picking the side that overlaps the least with other monitors'
  /// working areas (ties favor bottom-right).
  pub fn monitors_by_hide_corner(&self) -> Vec<(Monitor, HideCorner)> {
    const TEST_FRAME_SIZE: i32 = 400;
    const VISIBLE_SLIVER: i32 = 1;

    let monitors = self.monitors();
    let working_areas = monitors
      .iter()
      .map(|monitor| monitor.native_properties().working_area)
      .collect::<Vec<_>>();

    monitors
      .into_iter()
      .enumerate()
      .map(|(idx, monitor)| {
        let monitor_rect = &working_areas[idx];
        let test_frame_y = monitor_rect.bottom - TEST_FRAME_SIZE;

        let left_test_frame = Rect::from_xy(
          monitor_rect.left - TEST_FRAME_SIZE + VISIBLE_SLIVER,
          test_frame_y,
          TEST_FRAME_SIZE,
          TEST_FRAME_SIZE,
        );

        let right_test_frame = Rect::from_xy(
          monitor_rect.right - VISIBLE_SLIVER,
          test_frame_y,
          TEST_FRAME_SIZE,
          TEST_FRAME_SIZE,
        );

        let overlap_area = |test_frame: &Rect| -> i32 {
          working_areas
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, rect)| test_frame.intersection_area(rect))
            .sum()
        };

        let left_overlap = overlap_area(&left_test_frame);
        let right_overlap = overlap_area(&right_test_frame);

        let corner = if left_overlap < right_overlap {
          HideCorner::BottomLeft
        } else {
          HideCorner::BottomRight
        };

        (monitor, corner)
      })
      .collect()
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
  #[allow(clippy::too_many_lines)]
  pub fn workspace_by_target(
    &self,
    origin_workspace: &Workspace,
    target: WorkspaceTarget,
    config: &UserConfig,
  ) -> anyhow::Result<(Option<String>, Option<Workspace>)> {
    let (name, workspace) = match target {
      WorkspaceTarget::Name(name) => {
        #[allow(clippy::match_bool)]
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
          .and_then(|name| self.workspace_by_name(name)),
      ),
      WorkspaceTarget::NextActive => {
        let active_workspaces = self.sorted_workspaces(config);
        let origin_index = active_workspaces
          .iter()
          .position(|workspace| workspace.id() == origin_workspace.id())
          .context("Failed to get index of given workspace.")?;

        let next_active_workspace = active_workspaces
          .get(origin_index + 1)
          .or_else(|| active_workspaces.first());

        (
          next_active_workspace.map(|workspace| workspace.config().name),
          next_active_workspace.cloned(),
        )
      }
      WorkspaceTarget::PreviousActive => {
        let active_workspaces = self.sorted_workspaces(config);
        let origin_index = active_workspaces
          .iter()
          .position(|workspace| workspace.id() == origin_workspace.id())
          .context("Failed to get index of given workspace.")?;

        let prev_active_workspace = active_workspaces.get(
          origin_index
            .checked_sub(1)
            .unwrap_or(active_workspaces.len() - 1),
        );

        (
          prev_active_workspace.map(|workspace| workspace.config().name),
          prev_active_workspace.cloned(),
        )
      }
      WorkspaceTarget::NextActiveInMonitor => {
        let monitor = origin_workspace
          .monitor()
          .context("No monitor in workspace")?;

        let mut workspace_in_monitor = monitor.workspaces();
        config.sort_workspaces(&mut workspace_in_monitor);

        let origin_index = workspace_in_monitor
          .iter()
          .position(|workspace| workspace.id() == origin_workspace.id())
          .context("Failed to get index of give workspace")?;

        let next_active_workspace_in_monitor = workspace_in_monitor
          .get(origin_index + 1)
          .or_else(|| workspace_in_monitor.first());

        (
          next_active_workspace_in_monitor
            .map(|workspace| workspace.config().name),
          next_active_workspace_in_monitor.cloned(),
        )
      }
      WorkspaceTarget::PreviousActiveInMonitor => {
        let monitor = origin_workspace
          .monitor()
          .context("No monitor in workspace")?;

        let mut workspace_in_monitor = monitor.workspaces();
        config.sort_workspaces(&mut workspace_in_monitor);

        let origin_index = workspace_in_monitor
          .iter()
          .position(|workspace| workspace.id() == origin_workspace.id())
          .context("Failed to get index of give workspace")?;

        let prev_active_workspace_in_monitor = workspace_in_monitor.get(
          origin_index
            .checked_sub(1)
            .unwrap_or(workspace_in_monitor.len() - 1),
        );

        (
          prev_active_workspace_in_monitor
            .map(|workspace| workspace.config().name),
          prev_active_workspace_in_monitor.cloned(),
        )
      }
      WorkspaceTarget::Next => {
        let workspaces = &config.value.workspaces;
        let origin_name = origin_workspace.config().name.clone();
        let origin_index = workspaces
          .iter()
          .position(|workspace| workspace.name == origin_name)
          .context("Failed to get index of given workspace.")?;

        let next_workspace_config = workspaces
          .get(origin_index + 1)
          .or_else(|| workspaces.first());

        let next_workspace_name =
          next_workspace_config.map(|config| config.name.clone());

        let next_workspace = next_workspace_name
          .as_ref()
          .and_then(|name| self.workspace_by_name(name));

        (next_workspace_name, next_workspace)
      }
      WorkspaceTarget::Previous => {
        let workspaces = &config.value.workspaces;
        let origin_name = origin_workspace.config().name.clone();
        let origin_index = workspaces
          .iter()
          .position(|workspace| workspace.name == origin_name)
          .context("Failed to get index of given workspace.")?;

        let previous_workspace_config = workspaces.get(
          origin_index.checked_sub(1).unwrap_or(workspaces.len() - 1),
        );

        let previous_workspace_name =
          previous_workspace_config.map(|config| config.name.clone());

        let previous_workspace = previous_workspace_name
          .as_ref()
          .and_then(|name| self.workspace_by_name(name));

        (previous_workspace_name, previous_workspace)
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
    self
      .pending_sync
      .containers_to_redraw()
      .values()
      .flat_map(CommonGetters::self_and_descendants)
      .filter(|container| !container.is_detached())
      .filter_map(|container| container.try_into().ok())
      .collect()
  }

  /// Gets the currently focused container. This can either be a window or
  /// a workspace without any descendant windows.
  pub fn focused_container(&self) -> Option<Container> {
    self.root_container.descendant_focus_order().next()
  }

  /// Emits a WM event through an MSPC channel.
  ///
  /// Does not emit events while the WM is paused or populating initial
  /// state. This is to prevent events (e.g. workspace activation events)
  /// from being emitted via IPC server before the initial state is
  /// prepared.
  pub fn emit_event(&self, event: WmEvent) {
    if self.has_initialized
      && (!self.is_paused || matches!(event, WmEvent::PauseChanged { .. }))
    {
      if let Err(err) = self.event_tx.send(event) {
        warn!("Failed to send event: {}", err);
      }
    }
  }

  /// Starts graceful shutdown via an MSPC channel.
  pub fn emit_exit(&self) -> anyhow::Result<()> {
    self.exit_tx.send(())?;
    Ok(())
  }

  pub fn container_by_id(&self, id: Uuid) -> Option<Container> {
    self
      .root_container
      .self_and_descendants()
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
        matches!(
          (descendant.state(), removed_window.state()),
          (WindowState::Tiling, WindowState::Tiling)
            | (WindowState::Floating(_), WindowState::Floating(_))
            | (WindowState::Fullscreen(_), WindowState::Fullscreen(_))
        )
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

  /// Returns all containers that contain the given point.
  #[allow(clippy::unused_self)]
  pub fn containers_at_point(
    &self,
    origin_container: &Container,
    point: &Point,
  ) -> Vec<Container> {
    origin_container
      .descendants()
      .filter(|descendant| {
        descendant
          .to_rect()
          .is_ok_and(|rect| rect.contains_point(point))
      })
      .collect()
  }

  /// Returns the monitor that contains the given point.
  pub fn monitor_at_point(&self, point: &Point) -> Option<Monitor> {
    self
      .monitors()
      .iter()
      .find(|monitor| {
        monitor
          .to_rect()
          .is_ok_and(|rect| rect.contains_point(point))
      })
      .cloned()
  }

  /// Cleans up windows that are no longer alive.
  ///
  /// This addresses the "ghost window" issue where applications may
  /// terminate without sending window destroy events, leaving invalid
  /// windows in WM state.
  ///
  /// See: <https://github.com/glzr-io/glazewm/issues/1219>
  pub fn cleanup_invalid_windows(&mut self) -> anyhow::Result<()> {
    let invalid_windows = self
      .windows()
      .into_iter()
      .filter(|window| !window.native().is_valid());

    for window in invalid_windows {
      tracing::info!("Removing invalid window: {}", window);
      unmanage_window(window, self)?;
    }

    // Prune ignored windows that are no longer valid.
    self.ignored_windows.retain(NativeWindow::is_valid);

    self
      .native_windows_pending_remanage
      .retain(|entry| entry.window.is_valid());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn guard_expiring_in(duration: Duration) -> RemanageSnapGuard {
    RemanageSnapGuard {
      monitor_id: Uuid::nil(),
      expires_at: Instant::now() + duration,
    }
  }

  #[test]
  fn snap_guard_settles_on_target_monitor() {
    let guard = guard_expiring_in(Duration::from_secs(60));

    assert!(guard.is_settled(guard.monitor_id, true, Instant::now()));
  }

  #[test]
  fn snap_guard_settles_when_target_monitor_removed() {
    let guard = guard_expiring_in(Duration::from_secs(60));
    let other_monitor_id = Uuid::new_v4();

    assert!(guard.is_settled(other_monitor_id, false, Instant::now()));
  }

  #[test]
  fn snap_guard_settles_after_expiry() {
    let guard = guard_expiring_in(Duration::from_secs(60));
    let other_monitor_id = Uuid::new_v4();

    assert!(guard.is_settled(
      other_monitor_id,
      true,
      guard.expires_at + Duration::from_millis(1),
    ));
  }

  #[test]
  fn snap_guard_holds_while_unsettled() {
    let guard = guard_expiring_in(Duration::from_secs(60));
    let other_monitor_id = Uuid::new_v4();

    assert!(!guard.is_settled(other_monitor_id, true, Instant::now()));
  }

  fn mock_state() -> WmState {
    let (event_tx, _event_rx) = mpsc::unbounded_channel();
    let (exit_tx, _exit_rx) = mpsc::unbounded_channel();
    WmState::new(Dispatcher::mock(), event_tx, exit_tx)
  }

  #[test]
  fn display_change_not_settling_initially() {
    let state = mock_state();

    assert!(!state.is_display_change_settling());
  }

  #[test]
  fn display_change_settling_after_note() {
    let mut state = mock_state();
    state.note_display_change();

    assert!(state.is_display_change_settling());
  }
}

impl Drop for WmState {
  fn drop(&mut self) {
    let managed_windows = self.windows();

    for window in &managed_windows {
      // Redraw windows to their intended positions. On macOS, this will
      // unhide windows that are on other workspaces.
      if let Ok(rect) = window.to_rect() {
        if let Err(err) = window.native().set_frame(&rect) {
          warn!("Failed to redraw window on cleanup: {:?}", err);
        }
      }

      // Reset any effects on Windows.
      #[cfg(target_os = "windows")]
      {
        if let Err(err) = window.native().show() {
          warn!("Failed to show window: {:?}", err);
        }

        let _ = window.native().set_taskbar_visibility(true);
        let _ = window.native().set_border_color(None);
        let _ = window
          .native()
          .set_transparency(&OpacityValue::from_alpha(u8::MAX));
      }
    }
  }
}
