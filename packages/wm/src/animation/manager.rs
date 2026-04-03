use std::{
  collections::HashMap,
  sync::{atomic::AtomicBool, atomic::Ordering, Arc},
};

use tokio::sync::mpsc;
use uuid::Uuid;
use wm_platform::{NativeWindow, OpacityValue, Rect};
#[cfg(target_os = "windows")]
use wm_platform::{NativeSurrogate, NativeWindowWindowsExt};

use crate::{
  animation::state::WindowAnimationState,
  commands::general::platform_sync,
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Result of [`AnimationManager::start_animation_if_needed`], describing
/// what the caller should do with the real app window's position.
pub enum AnimationPositionResult {
  /// Apply this rect to the real window via `reposition_window`.
  Apply(Rect),
  /// The surrogate overlay is handling all visuals; skip repositioning the
  /// real window this frame.
  Frozen,
}

/// Manages animations for all windows.
pub struct AnimationManager {
  /// Active animations keyed by window ID.
  animations: HashMap<Uuid, WindowAnimationState>,
  /// Sender for animation tick events.
  animation_tick_tx: mpsc::UnboundedSender<()>,
  /// Whether the animation timer is currently running.
  animation_timer_running: Arc<AtomicBool>,
  /// Active surrogate overlay windows, keyed by window ID.
  ///
  /// A surrogate is created when a movement animation starts with
  /// `use_surrogate = true` and is destroyed once the animation completes
  /// and the real window has been moved to its final position.
  #[cfg(target_os = "windows")]
  surrogates: HashMap<Uuid, NativeSurrogate>,
  /// Surrogates that have been removed from `surrogates` after their
  /// animation completed, but must stay alive until after the final
  /// `platform_sync` call has repositioned the real window.
  #[cfg(target_os = "windows")]
  pending_surrogate_cleanup: Vec<NativeSurrogate>,
}

impl AnimationManager {
  pub fn new(animation_tick_tx: mpsc::UnboundedSender<()>) -> Self {
    Self {
      animations: HashMap::new(),
      animation_tick_tx,
      animation_timer_running: Arc::new(AtomicBool::new(false)),
      #[cfg(target_os = "windows")]
      surrogates: HashMap::new(),
      #[cfg(target_os = "windows")]
      pending_surrogate_cleanup: Vec::new(),
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

  /// Removes a window's animation and any associated surrogate.
  #[allow(dead_code)]
  pub fn remove_animation(&mut self, window_id: &Uuid) {
    self.animations.remove(window_id);
    #[cfg(target_os = "windows")]
    self.surrogates.remove(window_id);
  }

  /// Removes all completed animations and returns the list of window IDs
  /// that had animations complete.
  ///
  /// Surrogates for completed animations are moved to
  /// `pending_surrogate_cleanup` so they remain visible until after the
  /// final `platform_sync` call has repositioned the real windows.
  pub fn remove_completed_animations(&mut self) -> Vec<Uuid> {
    let completed_ids: Vec<Uuid> = self
      .animations
      .iter()
      .filter(|(_, anim)| anim.is_complete())
      .map(|(id, _)| *id)
      .collect();

    for id in &completed_ids {
      self.animations.remove(id);
      #[cfg(target_os = "windows")]
      if let Some(surrogate) = self.surrogates.remove(id) {
        self.pending_surrogate_cleanup.push(surrogate);
      }
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

  /// Clears all animations and any associated surrogates.
  #[allow(dead_code)]
  pub fn clear(&mut self) {
    self.animations.clear();
    #[cfg(target_os = "windows")]
    {
      self.surrogates.clear();
      self.pending_surrogate_cleanup.clear();
    }
  }

  /// Ensures the animation timer is running if there are active animations.
  pub fn ensure_timer_running(
    &self,
    state: &crate::wm_state::WmState,
    config: &UserConfig,
  ) {
    if self.has_active_animations()
      && !self.animation_timer_running.load(Ordering::Relaxed) {

      self.animation_timer_running.store(true, Ordering::Relaxed);
      let tx = self.animation_tick_tx.clone();
      let timer_flag = self.animation_timer_running.clone();

      // Calculate frame time based on monitor refresh rate, capped by
      // config max_frame_rate. Default to 60 FPS if refresh rate
      // cannot be determined.
      let mut frame_time_ms = 16u32;

      if let Some(container) = state.focused_container() {
        if let Some(monitor) = CommonGetters::monitor(&container) {
          // Default to 60Hz if refresh rate cannot be determined.
          let refresh_rate =
            monitor.native_properties().refresh_rate.unwrap_or(60);
          // Cap refresh rate at configured max_frame_rate.
          let capped_rate =
            refresh_rate.min(config.value.animations.max_frame_rate);
          // Enforce 60 Hz minimum for safety.
          let final_rate = capped_rate.max(60);
          // Convert refresh rate to milliseconds per frame.
          frame_time_ms = 1000 / final_rate;
        }
      } else {
        // If no monitor found, use max_frame_rate from config (capped at
        // 60Hz minimum).
        let final_rate = config.value.animations.max_frame_rate.max(60);
        frame_time_ms = 1000 / final_rate;
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
  #[allow(dead_code)] // Public API method, may be used externally.
  pub fn update(
    &mut self,
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    Self::update_internal(state, config)
  }

  /// Internal update method that accesses `animation_manager` through state.
  pub(crate) fn update_internal(
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    if !state.animation_manager.has_active_animations() {
      return Ok(());
    }

    // Collect windows whose animations are still in progress.
    let active_window_ids: Vec<_> = state
      .animation_manager
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

    // Remove completed animations. Their surrogates are moved to
    // `pending_surrogate_cleanup` and must outlive the `platform_sync`
    // call below so the real window is repositioned before the overlay
    // disappears.
    let completed_ids =
      state.animation_manager.remove_completed_animations();

    // Queue completed animations for final redraw so the real window is
    // moved to its target position.
    for window_id in &completed_ids {
      if let Some(container) = state.container_by_id(*window_id) {
        if let Ok(window) = container.as_window_container() {
          state.pending_sync.queue_container_to_redraw(window);
        }
      }
    }

    // Sync platform if there are changes.
    if state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    // Drop surrogates now that the real windows have been repositioned by
    // `platform_sync`. This ensures the overlay never disappears before
    // the underlying window is in its final position.
    #[cfg(target_os = "windows")]
    state.animation_manager.pending_surrogate_cleanup.clear();

    // Continue timer if there are still active animations.
    state.animation_manager.ensure_timer_running(state, config);

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
        // Don't restart animations that are completing or already at
        // target.
        if anim.is_complete() {
          false
        } else {
          // Check if target has changed significantly from the animation's
          // current target. Use a reasonable threshold to avoid
          // creating animations for every tiny change.
          let target_distance = (anim.target_rect.x() - target_rect.x())
            .abs()
            + (anim.target_rect.y() - target_rect.y()).abs()
            + (anim.target_rect.width() - target_rect.width()).abs()
            + (anim.target_rect.height() - target_rect.height()).abs();
          target_distance > threshold
        }
      } else if let Some(prev_target) = previous_target {
        // Compare previous target to new target, not current position to
        // target.
        let distance = (prev_target.x() - target_rect.x()).abs()
          + (prev_target.y() - target_rect.y()).abs()
          + (prev_target.width() - target_rect.width()).abs()
          + (prev_target.height() - target_rect.height()).abs();
        distance > threshold
      } else {
        // First time seeing this window, no animation needed.
        false
      }
    } else {
      false
    }
  }

  /// Starts an animation if needed and returns how the caller should handle
  /// the real window's position this frame.
  ///
  /// When a surrogate overlay is active the real window must not be
  /// repositioned (returns [`AnimationPositionResult::Frozen`]). Only when
  /// the animation has been removed from the map (i.e. the animation
  /// completed and `remove_completed_animations` was already called) does
  /// the caller receive [`AnimationPositionResult::Apply`] with the final
  /// target rect so the real window can be moved exactly once.
  pub fn start_animation_if_needed(
    &mut self,
    window_id: Uuid,
    is_opening: bool,
    target_rect: Rect,
    previous_target: Option<Rect>,
    // Only used on Windows to capture the source window for the surrogate.
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    native_window: &NativeWindow,
    config: &UserConfig,
  ) -> (AnimationPositionResult, Option<OpacityValue>) {
    let threshold =
      config.value.animations.window_move.threshold_px as i32;

    let existing_animation = self.get_animation(&window_id).cloned();

    let should_start = self.should_start_new_animation(
      &window_id,
      is_opening,
      &target_rect,
      previous_target.as_ref(),
      threshold,
      config,
    );

    if should_start {
      if is_opening {
        let animation = WindowAnimationState::new_open(
          target_rect.clone(),
          &config.value.animations.window_open,
        );
        self.start_animation(window_id, animation);
      } else if let Some(prev_target) = previous_target {
        // Start from the current animated position if an animation is
        // already in progress (cancel-and-replace).
        let start_rect = if let Some(existing_anim) = &existing_animation {
          existing_anim.current_rect()
        } else {
          prev_target
        };

        let animation = WindowAnimationState::new_movement(
          start_rect.clone(),
          target_rect.clone(),
          &config.value.animations.window_move,
        );
        self.start_animation(window_id, animation);

        // Create a surrogate if configured and one doesn't already exist
        // for this window. On cancel-and-replace the existing surrogate is
        // reused so we avoid a redundant capture.
        #[cfg(target_os = "windows")]
        if config.value.animations.window_move.use_surrogate
          && !self.surrogates.contains_key(&window_id)
        {
          match NativeSurrogate::create(
            native_window.hwnd(),
            &start_rect,
          ) {
            Ok(surrogate) => {
              self.surrogates.insert(window_id, surrogate);
            }
            Err(err) => {
              tracing::warn!(
                "Failed to create surrogate for window {window_id}: \
                 {err}. Falling back to direct animation."
              );
            }
          }
        }
      }
    }

    // Re-fetch the animation after potentially starting a new one.
    if let Some(animation) = self.get_animation(&window_id) {
      let current_rect = animation.current_rect();
      let opacity = animation.current_opacity();

      // If a surrogate is active, update it to the current interpolated
      // rect and tell the caller to leave the real window untouched.
      #[cfg(target_os = "windows")]
      if let Some(surrogate) = self.surrogates.get(&window_id) {
        if let Err(err) = surrogate.update(&current_rect) {
          tracing::warn!(
            "Failed to update surrogate for window {window_id}: {err}."
          );
        }
        return (
          AnimationPositionResult::Frozen,
          None,
        );
      }

      (AnimationPositionResult::Apply(current_rect), opacity)
    } else {
      // No animation in the map — animation has completed and
      // `remove_completed_animations` was already called. Apply the final
      // target rect to the real window.
      (AnimationPositionResult::Apply(target_rect), None)
    }
  }
}

