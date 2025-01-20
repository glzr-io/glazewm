use std::collections::HashMap;

use uuid::Uuid;

use crate::{models::Container, traits::CommonGetters};

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct PendingSync {
  /// Containers (and their descendants) that have a pending redraw.
  containers_to_redraw: HashMap<Uuid, Container>,

  /// Whether native focus should be reassigned to the WM's focused
  /// container.
  focus_change: bool,

  /// Whether window effect for the focused window should be updated.
  update_focused_window_effect: bool,

  /// Whether window effects for all windows should be updated.
  update_all_window_effects: bool,

  /// Whether to jump the cursor to the focused container (if enabled in
  /// user config).
  cursor_jump: bool,
}

impl PendingSync {
  pub fn has_changes(&self) -> bool {
    self.focus_change
      || self.update_focused_window_effect
      || self.update_all_window_effects
      || self.cursor_jump
      || !self.containers_to_redraw.is_empty()
  }

  pub fn clear(&mut self) -> &mut Self {
    self.containers_to_redraw.clear();
    self.focus_change = false;
    self.update_focused_window_effect = false;
    self.update_all_window_effects = false;
    self.cursor_jump = false;
    self
  }

  pub fn add_container_to_redraw<T>(&mut self, container: T) -> &mut Self
  where
    T: Into<Container>,
  {
    let container: Container = container.into();
    self.containers_to_redraw.insert(container.id(), container);
    self
  }

  pub fn add_containers_to_redraw<I, T>(
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

  pub fn remove_container_from_redraw<T>(
    &mut self,
    container: T,
  ) -> &mut Self
  where
    T: Into<Container>,
  {
    self.containers_to_redraw.remove(&container.into().id());
    self
  }

  pub fn mark_focus_change(&mut self) -> &mut Self {
    self.focus_change = true;
    self
  }

  pub fn mark_update_focused_window_effect(&mut self) -> &mut Self {
    self.update_focused_window_effect = true;
    self
  }

  pub fn mark_update_all_window_effects(&mut self) -> &mut Self {
    self.update_all_window_effects = true;
    self
  }

  pub fn mark_cursor_jump(&mut self) -> &mut Self {
    self.cursor_jump = true;
    self
  }

  pub fn focus_change(&self) -> bool {
    self.focus_change
  }

  pub fn update_focused_window_effect(&self) -> bool {
    self.update_focused_window_effect
  }

  pub fn update_all_window_effects(&self) -> bool {
    self.update_all_window_effects
  }

  pub fn cursor_jump(&self) -> bool {
    self.cursor_jump
  }

  pub fn containers_to_redraw(&self) -> &HashMap<Uuid, Container> {
    &self.containers_to_redraw
  }
}
