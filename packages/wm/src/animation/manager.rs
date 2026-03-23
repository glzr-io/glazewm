use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use tokio::sync::mpsc;
use uuid::Uuid;
use wm_platform::{OpacityValue, Rect};

use crate::{
  animation::state::WindowAnimationState,
  commands::general::platform_sync, traits::CommonGetters,
  user_config::UserConfig, wm_state::WmState,
};

/// Result of starting an animation, indicating how the caller should
/// handle the real window.
pub struct AnimationStartResult {
  /// The rect to use for the real window. When an overlay is active this
  /// is the target rect (real window is hidden). Otherwise it is the
  /// interpolated animated rect.
  pub rect: Rect,
  /// Optional opacity override for the real window. Used on Windows when
  /// no overlay is active.
  pub opacity: Option<OpacityValue>,
  /// Whether an overlay is handling the visual animation. When `true`,
  /// the caller should hide the real window.
  pub has_overlay: bool,
}

/// Manages animations for all windows.
pub struct AnimationManager {
  /// Active animations keyed by window ID.
  animations: HashMap<Uuid, WindowAnimationState>,
  /// Sender for animation tick events.
  animation_tick_tx: mpsc::UnboundedSender<()>,
  /// Whether the animation timer is currently running.
  animation_timer_running: Arc<AtomicBool>,

  /// Overlay windows for screenshot-based animations.
  overlays: HashMap<Uuid, wm_platform::OverlayWindow>,
}

impl AnimationManager {
  pub fn new(animation_tick_tx: mpsc::UnboundedSender<()>) -> Self {
    Self {
      animations: HashMap::new(),
      animation_tick_tx,
      animation_timer_running: Arc::new(AtomicBool::new(false)),
      overlays: HashMap::new(),
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
  pub fn get_animation(
    &self,
    window_id: &Uuid,
  ) -> Option<&WindowAnimationState> {
    self.animations.get(window_id)
  }

  /// Removes a window's animation.
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

  /// Ensures the animation timer is running if there are active
  /// animations.
  pub fn ensure_timer_running(
    &self,
    state: &crate::wm_state::WmState,
    config: &UserConfig,
  ) {
    if self.has_active_animations()
      && !self.animation_timer_running.load(Ordering::Relaxed)
    {
      self.animation_timer_running.store(true, Ordering::Relaxed);
      let tx = self.animation_tick_tx.clone();
      let timer_flag = self.animation_timer_running.clone();

      // Calculate frame time based on monitor refresh rate, capped by
      // config max_frame_rate Default to 60 FPS if refresh rate
      // cannot be determined
      let mut frame_time_ms = 16u32;

      if let Some(container) = state.focused_container() {
        if let Some(monitor) = CommonGetters::monitor(&container) {
          // Default to 60Hz if refresh rate cannot be determined
          let refresh_rate =
            monitor.native_properties().refresh_rate.unwrap_or(60);
          // Cap refresh rate at configured max_frame_rate
          let capped_rate =
            refresh_rate.min(config.value.animations.max_frame_rate);
          // Cap at 60 Hz minimum for safety
          let final_rate = capped_rate.max(60);
          // Convert refresh rate to milliseconds per frame
          frame_time_ms = 1000 / final_rate;
        }
      } else {
        // If no monitor found, use max_frame_rate from config (capped at
        // 60Hz minimum)
        let final_rate = config.value.animations.max_frame_rate.max(60);
        frame_time_ms = 1000 / final_rate;
      }

      tokio::spawn(async move {
        let mut interval = tokio::time::interval(
          tokio::time::Duration::from_millis(u64::from(frame_time_ms)),
        );
        interval
          .set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

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
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    if !state.animation_manager.has_active_animations() {
      return Ok(());
    }

    // Update active overlay positions directly without moving real
    // windows.
    let active_ids: Vec<_> = state
      .animation_manager
      .active_window_ids()
      .into_iter()
      .filter(|id| {
        state
          .animation_manager
          .get_animation(id)
          .is_some_and(|anim| !anim.is_complete())
      })
      .collect();

    let updates: Vec<_> = active_ids
      .iter()
      .filter_map(|id| {
        let anim = state.animation_manager.get_animation(id)?;
        let overlay = state.animation_manager.overlays.get(id)?;
        Some((overlay, anim.current_rect(), anim.current_opacity()))
      })
      .collect();

    let batch: Vec<_> = updates
      .iter()
      .map(|(overlay, rect, opacity)| (*overlay, rect, *opacity))
      .collect();

    wm_platform::move_group(&batch);

    // Remove completed animations and queue their windows for a final
    // redraw at the target position.
    let completed_ids =
      state.animation_manager.remove_completed_animations();

    for window_id in &completed_ids {
      if let Some(container) = state.container_by_id(*window_id) {
        if let Ok(window) = container.as_window_container() {
          state.pending_sync.queue_container_to_redraw(window);
        }
      }
    }

    // Sync platform for completed animations (final positioning).
    if state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    // Destroy overlays after the real windows have been repositioned so
    // there is no visible gap.
    for window_id in &completed_ids {
      if let Some(overlay) =
        state.animation_manager.overlays.remove(window_id)
      {
        if let Err(err) = overlay.destroy() {
          tracing::warn!("Failed to destroy overlay: {}", err);
        }
      }
    }

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
        // target
        if anim.is_complete() {
          false
        } else {
          // Check if target has changed significantly from the animation's
          // current target Use a reasonable threshold to avoid
          // creating animations for every tiny change
          let target_distance = (anim.target_rect.x() - target_rect.x())
            .abs()
            + (anim.target_rect.y() - target_rect.y()).abs()
            + (anim.target_rect.width() - target_rect.width()).abs()
            + (anim.target_rect.height() - target_rect.height()).abs();
          target_distance > threshold
        }
      } else if let Some(prev_target) = previous_target {
        // Compare PREVIOUS target to NEW target, not current position to
        // target
        let distance = (prev_target.x() - target_rect.x()).abs()
          + (prev_target.y() - target_rect.y()).abs()
          + (prev_target.width() - target_rect.width()).abs()
          + (prev_target.height() - target_rect.height()).abs();
        distance > threshold
      } else {
        // First time seeing this window, no animation needed
        false
      }
    } else {
      false
    }
  }

  /// Starts an animation if needed and returns information about how the
  /// caller should handle the real window.
  #[allow(clippy::too_many_arguments)]
  pub fn start_animation_if_needed(
    &mut self,
    window_id: Uuid,
    is_opening: bool,
    target_rect: Rect,
    previous_target: Option<Rect>,
    config: &UserConfig,
    native_window_id: wm_platform::WindowId,
    dispatcher: &wm_platform::Dispatcher,
  ) -> AnimationStartResult {
    #[allow(clippy::cast_possible_wrap)]
    let threshold =
      config.value.animations.window_move.threshold_px as i32;

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
        // Cancel and replace: start from current animated position if an
        // animation is running
        let start_rect = if let Some(existing_anim) = &existing_animation {
          existing_anim.current_rect()
        } else {
          prev_target
        };

        let animation_config = config.value.animations.window_move.clone();

        // Create animation from current position to new target (cancel and
        // replace)
        let animation = WindowAnimationState::new_movement(
          start_rect,
          target_rect.clone(),
          &animation_config,
        );
        self.start_animation(window_id, animation);
      }

      // Create an overlay window for the new animation.
      {
        // Destroy any existing overlay for this window.
        if let Some(old_overlay) = self.overlays.remove(&window_id) {
          if let Err(err) = old_overlay.destroy() {
            tracing::warn!("Failed to destroy old overlay: {}", err);
          }
        }

        // Get the initial rect from the animation we just started.
        let initial_rect = self.get_animation(&window_id).map_or_else(
          || target_rect.clone(),
          WindowAnimationState::current_rect,
        );

        match wm_platform::OverlayWindow::new(
          native_window_id,
          &initial_rect,
          dispatcher,
        ) {
          Ok(overlay) => {
            // Set initial opacity for fade animations.
            if let Some(anim) = self.get_animation(&window_id) {
              if let Some(opacity) = anim.current_opacity() {
                let _ = overlay.set_opacity(opacity.to_f32());
              }
            }
            self.overlays.insert(window_id, overlay);
          }
          Err(err) => {
            tracing::warn!("Failed to create overlay window: {}", err);
          }
        }
      }
    }

    // Get the current animation state (re-fetch after potentially starting
    // new animation)
    if let Some(animation) = self.get_animation(&window_id) {
      let has_overlay = self.overlays.contains_key(&window_id);

      AnimationStartResult {
        rect: if has_overlay {
          // With an overlay the real window is hidden, so move it to the
          // target immediately.
          target_rect
        } else {
          animation.current_rect()
        },
        opacity: animation.current_opacity(),
        has_overlay,
      }
    } else {
      AnimationStartResult {
        rect: target_rect,
        opacity: None,
        has_overlay: false,
      }
    }
  }

  /// Cancels any active overlay for the given window and destroys it.
  pub fn cancel_overlay(&mut self, window_id: &Uuid) {
    if let Some(overlay) = self.overlays.remove(window_id) {
      if let Err(err) = overlay.destroy() {
        tracing::warn!("Failed to destroy overlay on cancel: {}", err);
      }
    }
  }
}
