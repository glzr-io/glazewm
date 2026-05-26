use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::{
  models::{Container, Workspace},
  traits::CommonGetters,
};

#[derive(Debug, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct PendingSync {
  /// Containers (and their descendants) that have a pending redraw.
  containers_to_redraw: HashMap<Uuid, Container>,

  /// Workspaces where z-order should be updated. Windows that match the
  /// focused window's state should be brought to the front.
  workspaces_to_reorder: Vec<Workspace>,

  /// Whether native focus should be reassigned to the WM's focused
  /// container.
  needs_focus_update: bool,

  /// Whether window effect for the focused window should be updated.
  needs_focused_effect_update: bool,

  /// Whether window effects for all windows should be updated.
  needs_all_effects_update: bool,

  /// Whether to jump the cursor to the focused container (if enabled in
  /// user config).
  needs_cursor_jump: bool,

  /// Window IDs on the incoming workspace that should slide in.
  workspace_switch_incoming: HashSet<Uuid>,

  /// Window IDs on the outgoing workspace that should slide out.
  workspace_switch_outgoing: HashSet<Uuid>,

  /// Slide direction for the current workspace switch.
  ///
  /// `+1` means the target workspace has a higher config index (incoming
  /// slides in from the right, outgoing slides out to the left). `-1` means
  /// the opposite. `0` means no directional preference (fade in place).
  workspace_switch_direction: i32,

  /// Window IDs that just underwent a tiling/floating state change this sync
  /// cycle. Used by `platform_sync` to allow `window_move` animations across
  /// the state boundary (e.g. tiling → floating or floating → tiling).
  window_state_changes: HashSet<Uuid>,

  /// Window that should receive a focus-change animation this sync cycle.
  ///
  /// Set by `handle_window_focused` when focus moves to a managed window
  /// and the `focus_change` animation is enabled. Consumed by `platform_sync`
  /// when it processes that window's frame so `effect_opacity` is available.
  focus_animation_window: Option<Uuid>,
}

impl PendingSync {
  pub fn has_changes(&self) -> bool {
    !self.containers_to_redraw.is_empty()
      || !self.workspaces_to_reorder.is_empty()
      || self.needs_focus_update
      || self.needs_focused_effect_update
      || self.needs_all_effects_update
      || self.needs_cursor_jump
  }

  pub fn clear(&mut self) -> &mut Self {
    self.containers_to_redraw.clear();
    self.workspaces_to_reorder.clear();
    self.needs_focus_update = false;
    self.needs_focused_effect_update = false;
    self.needs_all_effects_update = false;
    self.needs_cursor_jump = false;
    self.workspace_switch_incoming.clear();
    self.workspace_switch_outgoing.clear();
    self.workspace_switch_direction = 0;
    self.window_state_changes.clear();
    self.focus_animation_window = None;
    self
  }

  pub fn queue_container_to_redraw<T>(&mut self, container: T) -> &mut Self
  where
    T: Into<Container>,
  {
    let container: Container = container.into();
    self.containers_to_redraw.insert(container.id(), container);
    self
  }

  pub fn queue_containers_to_redraw<I, T>(
    &mut self,
    containers: I,
  ) -> &mut Self
  where
    I: IntoIterator<Item = T>,
    T: Into<Container>,
  {
    for container in containers {
      let container: Container = container.into();
      self.containers_to_redraw.insert(container.id(), container);
    }

    self
  }

  pub fn dequeue_container_from_redraw<T>(
    &mut self,
    container: T,
  ) -> &mut Self
  where
    T: Into<Container>,
  {
    self.containers_to_redraw.remove(&container.into().id());
    self
  }

  pub fn queue_workspace_to_reorder(
    &mut self,
    workspace: Workspace,
  ) -> &mut Self {
    self.workspaces_to_reorder.push(workspace);
    self
  }

  pub fn queue_focus_change(&mut self) -> &mut Self {
    self.needs_focus_update = true;
    self
  }

  pub fn queue_focused_effect_update(&mut self) -> &mut Self {
    self.needs_focused_effect_update = true;
    self
  }

  pub fn queue_all_effects_update(&mut self) -> &mut Self {
    self.needs_all_effects_update = true;
    self
  }

  pub fn queue_cursor_jump(&mut self) -> &mut Self {
    self.needs_cursor_jump = true;
    self
  }

  pub fn needs_focus_update(&self) -> bool {
    self.needs_focus_update
  }

  pub fn needs_focused_effect_update(&self) -> bool {
    self.needs_focused_effect_update
  }

  pub fn needs_all_effects_update(&self) -> bool {
    self.needs_all_effects_update
  }

  pub fn needs_cursor_jump(&self) -> bool {
    self.needs_cursor_jump
  }

  pub fn containers_to_redraw(&self) -> &HashMap<Uuid, Container> {
    &self.containers_to_redraw
  }

  pub fn workspaces_to_reorder(&self) -> &Vec<Workspace> {
    &self.workspaces_to_reorder
  }

  /// Marks a window as having just changed tiling/floating state this cycle.
  pub fn mark_window_state_change(&mut self, id: Uuid) -> &mut Self {
    self.window_state_changes.insert(id);
    self
  }

  /// Returns `true` if the window changed tiling/floating state this cycle.
  pub fn is_window_state_change(&self, id: &Uuid) -> bool {
    self.window_state_changes.contains(id)
  }

  /// Schedules a focus-change animation for `id` this sync cycle.
  pub fn queue_focus_animation(&mut self, id: Uuid) -> &mut Self {
    self.focus_animation_window = Some(id);
    self
  }

  /// Consumes and returns the focus-animation window for this sync cycle.
  pub fn take_focus_animation(&mut self) -> Option<Uuid> {
    self.focus_animation_window.take()
  }

  /// Registers a window as an incoming workspace-switch target.
  pub fn setup_workspace_switch_incoming(
    &mut self,
    window_id: Uuid,
  ) -> &mut Self {
    self.workspace_switch_incoming.insert(window_id);
    self
  }

  /// Registers a window as an outgoing workspace-switch window.
  pub fn setup_workspace_switch_outgoing(
    &mut self,
    window_id: Uuid,
  ) -> &mut Self {
    self.workspace_switch_outgoing.insert(window_id);
    self
  }

  /// Returns `true` if the window is an incoming workspace-switch window.
  pub fn is_workspace_switch_incoming(&self, window_id: &Uuid) -> bool {
    self.workspace_switch_incoming.contains(window_id)
  }

  /// Returns `true` if the window is an outgoing workspace-switch window.
  pub fn is_workspace_switch_outgoing(&self, window_id: &Uuid) -> bool {
    self.workspace_switch_outgoing.contains(window_id)
  }

  /// Sets the slide direction for the current workspace switch.
  pub fn set_workspace_switch_direction(
    &mut self,
    direction: i32,
  ) -> &mut Self {
    self.workspace_switch_direction = direction;
    self
  }

  /// Returns the slide direction for the current workspace switch.
  pub fn workspace_switch_direction(&self) -> i32 {
    self.workspace_switch_direction
  }
}
