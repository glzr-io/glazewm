use std::{
  collections::HashMap,
  sync::{atomic::AtomicBool, atomic::Ordering, Arc},
};

use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::OpacityValue;
use wm_common::Rect;

use crate::{
  animation::state::WindowAnimationState,
  commands::general::platform_sync,
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Manages animations for all windows.
pub struct AnimationManager {
  /// Active animations keyed by window ID.
  animations: HashMap<Uuid, WindowAnimationState>,
  /// Sender for animation tick events.
  animation_tick_tx: mpsc::UnboundedSender<()>,
  /// Whether the animation timer is currently running.
  animation_timer_running: Arc<AtomicBool>,
}

impl AnimationManager {
  pub fn new(animation_tick_tx: mpsc::UnboundedSender<()>) -> Self {
    Self {
      animations: HashMap::new(),
      animation_tick_tx,
      animation_timer_running: Arc::new(AtomicBool::new(false)),
    }
  }

  /// Starts a new animation for a window.
  pub fn start_animation(
    &mut self,
    window_id: Uuid,
    animation: WindowAnimationState,
  ) {
    self.animations.insert(window_id, animation);
  }

  /// Gets the current state of a window's animation.
  pub fn get_animation(&self, window_id: &Uuid) -> Option<&WindowAnimationState> {
    self.animations.get(window_id)
  }

  /// Removes a window's animation.
  #[allow(dead_code)]
  pub fn remove_animation(&mut self, window_id: &Uuid) {
    self.animations.remove(window_id);
  }

  /// Removes all completed animations and returns the list of window IDs
  /// that had animations complete.
  pub fn remove_completed_animations(&mut self) -> Vec<Uuid> {
    let completed_ids: Vec<Uuid> = self
      .animations
      .iter()
      .filter(|(_, anim)| anim.is_complete())
      .map(|(id, _)| *id)
      .collect();

    for id in &completed_ids {
      self.animations.remove(id);
    }

    completed_ids
  }

  /// Whether there are any active animations.
  pub fn has_active_animations(&self) -> bool {
    !self.animations.is_empty()
  }

  /// Gets all active animation window IDs.
  pub fn active_window_ids(&self) -> Vec<Uuid> {
    self.animations.keys().copied().collect()
  }

  /// Clears all animations.
  #[allow(dead_code)]
  pub fn clear(&mut self) {
    self.animations.clear();
  }

  /// Ensures the animation timer is running if there are active animations.
  pub fn ensure_timer_running(&self, state: &crate::wm_state::WmState) {
    if self.has_active_animations()
      && !self.animation_timer_running.load(Ordering::Relaxed) {

      self.animation_timer_running.store(true, Ordering::Relaxed);
      let tx = self.animation_tick_tx.clone();
      let timer_flag = self.animation_timer_running.clone();

      // Calculate frame time based on monitor refresh rate
      // Default to 60 FPS if refresh rate cannot be determined
      let mut frame_time_ms = 16u32;

      if let Some(container) = state.focused_container() {
        if let Some(monitor) = CommonGetters::monitor(&container) {
          // Default to 60Hz if refresh rate cannot be determined
          let refresh_rate = monitor.native().refresh_rate().unwrap_or(60);
          // Convert refresh rate to milliseconds per frame
          // Cap at 60 Hz minimum for safety
          frame_time_ms = 1000 / refresh_rate.max(60);
        }
      }

      tokio::spawn(async move {
        let mut interval = tokio::time::interval(
          tokio::time::Duration::from_millis(frame_time_ms as u64)
        );
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
          interval.tick().await;
          if tx.send(()).is_err() {
            break;
          }
        }

        timer_flag.store(false, Ordering::Relaxed);
      });
    }
  }

  /// Updates all active animations and redraws windows that are animating.
  #[allow(dead_code)] // Public API method, may be used externally
  pub fn update(
    &mut self,
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    Self::update_internal(state, config)
  }

  /// Internal update method that accesses animation_manager through state.
  pub(crate) fn update_internal(
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    if !state.animation_manager.has_active_animations() {
      return Ok(());
    }

    // Get windows that have INCOMPLETE animations only
    let active_window_ids: Vec<_> = state.animation_manager
      .active_window_ids()
      .into_iter()
      .filter(|id| {
        state.animation_manager
          .get_animation(id)
          .map(|anim| !anim.is_complete())
          .unwrap_or(false)
      })
      .collect();

    for window_id in &active_window_ids {
      if let Some(container) = state.container_by_id(*window_id) {
        if let Ok(window) = container.as_window_container() {
          state.pending_sync.queue_container_to_redraw(window);
        }
      }
    }

    // Remove completed animations and get their IDs
    let completed_ids = state.animation_manager.remove_completed_animations();

    // Queue completed animations for final redraw
    for window_id in &completed_ids {
      if let Some(container) = state.container_by_id(*window_id) {
        if let Ok(window) = container.as_window_container() {
          state.pending_sync.queue_container_to_redraw(window);
        }
      }
    }

    // Sync platform if there are changes
    if state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    // Continue timer if there are still active animations
    state.animation_manager.ensure_timer_running(state);

    Ok(())
  }

  /// Determines whether a new animation should be started for a window.
  fn should_start_new_animation(
    &self,
    window_id: &Uuid,
    is_opening: bool,
    target_rect: &Rect,
    previous_target: Option<&Rect>,
    threshold: i32,
    config: &UserConfig,
  ) -> bool {
    let existing_animation = self.get_animation(window_id);

    if is_opening && config.value.animations.window_open.enabled {
      existing_animation.is_none()
    } else if !is_opening && config.value.animations.window_move.enabled {
      if let Some(anim) = existing_animation {
        // Don't restart animations that are completing or already at target
        if anim.is_complete() {
          false
        } else {
          // Check if target has changed significantly from the animation's current target
          // Use a reasonable threshold to avoid creating animations for every tiny change
          let target_distance = (anim.target_rect.x() - target_rect.x()).abs() +
                               (anim.target_rect.y() - target_rect.y()).abs() +
                               (anim.target_rect.width() - target_rect.width()).abs() +
                               (anim.target_rect.height() - target_rect.height()).abs();
          target_distance > threshold
        }
      } else if let Some(prev_target) = previous_target {
        // Compare PREVIOUS target to NEW target, not current position to target
        let distance = (prev_target.x() - target_rect.x()).abs() +
                       (prev_target.y() - target_rect.y()).abs() +
                       (prev_target.width() - target_rect.width()).abs() +
                       (prev_target.height() - target_rect.height()).abs();
        distance > threshold
      } else {
        // First time seeing this window, no animation needed
        false
      }
    } else {
      false
    }
  }

  /// Starts an animation if needed and returns the current animation state.
  /// Returns (rect, opacity) tuple.
  pub fn start_animation_if_needed(
    &mut self,
    window_id: Uuid,
    is_opening: bool,
    target_rect: Rect,
    previous_target: Option<Rect>,
    config: &UserConfig,
  ) -> (Rect, Option<OpacityValue>) {
    let threshold = config.value.animations.window_move.threshold_px as i32;

    // Check if there's already an animation for this window
    let existing_animation = self.get_animation(&window_id).cloned();

    // Decide whether to start a new animation
    let should_start = self.should_start_new_animation(
      &window_id,
      is_opening,
      &target_rect,
      previous_target.as_ref(),
      threshold,
      config,
    );

    // Start new animation if needed
    if should_start {
      if is_opening {
        let animation = WindowAnimationState::new_open(
          target_rect.clone(),
          &config.value.animations.window_open,
        );
        self.start_animation(window_id, animation);
      } else if let Some(prev_target) = previous_target {
        // Determine the start position for the new animation
        // Cancel and replace: start from current animated position if an animation is running
        let start_rect = if let Some(existing_anim) = &existing_animation {
          existing_anim.current_rect()
        } else {
          prev_target
        };

        let is_cancel_and_replace = existing_animation.is_some();

        // Choose animation config based on whether this is a cancel-and-replace
        let animation_config = if is_cancel_and_replace {
          // Use fixed short duration for interrupted animations to ensure consistent timing
          let mut movement_config = config.value.animations.window_move.clone();
          movement_config.duration_ms = 100; // Fixed 100ms for cancel-and-replace
          movement_config
        } else {
          // Use config duration directly
          config.value.animations.window_move.clone()
        };

        // Create animation from current position to new target (cancel and replace)
        let animation = WindowAnimationState::new_movement(
          start_rect,
          target_rect.clone(),
          &animation_config,
        );
        self.start_animation(window_id, animation);
      }
    }

    // Get the current animation state (re-fetch after potentially starting new animation)
    if let Some(animation) = self.get_animation(&window_id) {
      (animation.current_rect(), animation.current_opacity())
    } else {
      (target_rect, None)
    }
  }
}

