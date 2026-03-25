use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  time::Instant,
};

use tokio::sync::mpsc;
use uuid::Uuid;
use wm_platform::Rect;

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

  /// Shared animation surface with one layer per animating window.
  surface: Option<wm_platform::AnimationSurface>,

  /// Maps window UUIDs to their layer handles within the surface.
  layer_ids: HashMap<Uuid, wm_platform::LayerId>,
}

impl AnimationManager {
  pub fn new(animation_tick_tx: mpsc::UnboundedSender<()>) -> Self {
    Self {
      animations: HashMap::new(),
      animation_tick_tx,
      animation_timer_running: Arc::new(AtomicBool::new(false)),
      surface: None,
      layer_ids: HashMap::new(),
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

  /// Ensures the animation timer is running if there are active
  /// animations.
  pub fn ensure_timer_running(
    &self,
    state: &crate::wm_state::WmState,
    config: &UserConfig,
  ) {
    if self.animation_timer_running.load(Ordering::Relaxed) {
      return;
    }

    self.animation_timer_running.store(true, Ordering::Relaxed);
    let tx = self.animation_tick_tx.clone();
    let timer_flag = self.animation_timer_running.clone();

    let refresh_rate = state
      .focused_container()
      .and_then(|c| CommonGetters::monitor(&c))
      .and_then(|m| m.native_properties().refresh_rate)
      .unwrap_or(config.value.animations.max_frame_rate);

    let frame_rate = refresh_rate
      .min(config.value.animations.max_frame_rate)
      .max(60);

    // Convert refresh rate to milliseconds per frame.
    let frame_time_ms = 1000 / frame_rate;

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

  // ── Platform-specific overlay helpers
  // ──────────────────────────────────

  /// Returns whether a visual overlay exists for the given window.
  fn has_overlay(&self, window_id: &Uuid) -> bool {
    self.layer_ids.contains_key(window_id)
  }

  /// Creates or replaces the visual overlay for a window animation.
  ///
  /// Reuses the existing `AnimationSurface` if available, showing it
  /// again if it was hidden. Only creates a new surface on first use.
  fn create_overlay(
    &mut self,
    window_id: Uuid,
    native_window_id: wm_platform::WindowId,
    initial_rect: &Rect,
    opacity: Option<f32>,
    dispatcher: &wm_platform::Dispatcher,
  ) {
    // Remove any existing layer for this window.
    self.destroy_overlay(&window_id);

    // Reuse the existing surface, or create one on first use.
    if let Some(surface) = &self.surface {
      // Surface exists but may be hidden — show it.
      if let Err(err) = surface.show() {
        tracing::warn!("Failed to show animation surface: {}", err);
      }
    } else {
      match wm_platform::AnimationSurface::new(dispatcher) {
        Ok(surface) => self.surface = Some(surface),
        Err(err) => {
          tracing::warn!("Failed to create animation surface: {}", err);
          return;
        }
      }
    }

    let surface = self.surface.as_mut().expect("surface must exist");

    match surface.add_layer(native_window_id, initial_rect, opacity) {
      Ok(layer_id) => {
        self.layer_ids.insert(window_id, layer_id);
      }
      Err(err) => {
        tracing::warn!("Failed to add animation layer: {}", err);
      }
    }
  }

  /// Removes the visual overlay for a window.
  fn destroy_overlay(&mut self, window_id: &Uuid) {
    if let Some(layer_id) = self.layer_ids.remove(window_id) {
      if let Some(surface) = &mut self.surface {
        if let Err(err) = surface.remove_layer(layer_id) {
          tracing::warn!("Failed to remove animation layer: {}", err);
        }
      }
    }
  }

  /// Sends updated positions and opacities to the visual overlays for
  /// all in-progress animations.
  fn update_overlays(&self, active_ids: &[Uuid]) {
    let updates: Vec<_> = active_ids
      .iter()
      .filter_map(|id| {
        let anim = self.get_animation(id)?;
        let layer_id = self.layer_ids.get(id)?;
        Some((*layer_id, anim.current_rect(), anim.current_opacity()))
      })
      .collect();

    if let Some(surface) = &self.surface {
      if let Err(err) = surface.update_layers(updates) {
        tracing::warn!("Failed to update animation layers: {}", err);
      }
    }
  }

  /// Hides the shared surface when no layers remain.
  ///
  /// The surface is kept alive for reuse. On Windows, `hide` is a no-op
  /// so this call is harmless.
  fn hide_surface_if_empty(&mut self) {
    if self.layer_ids.is_empty() {
      if let Some(surface) = &self.surface {
        if let Err(err) = surface.hide() {
          tracing::warn!("Failed to hide animation surface: {}", err);
        }
      }
    }
  }

  // ── Core update + animation start
  // ──────────────────────────────────

  /// Updates all active animations and redraws windows that are animating.
  pub fn update(
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    if !state.animation_manager.has_active_animations() {
      return Ok(());
    }

    // Collect IDs of in-progress (not yet complete) animations.
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

    // Update overlay positions directly without moving real windows.
    state.animation_manager.update_overlays(&active_ids);

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

      // Briefly keep overlay up to hide flicker during sync.
      // TODO: This shouldn't block the main thread.
      std::thread::sleep(std::time::Duration::from_millis(20));

      // Destroy overlays after the real windows have been repositioned so
      // there is no visible gap.
      for window_id in &completed_ids {
        state.animation_manager.destroy_overlay(window_id);
      }

      // Hide the shared surface when all layers are gone (kept alive for
      // reuse).
      state.animation_manager.hide_surface_if_empty();
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
      if let Some(animation) = existing_animation {
        // Don't restart animations that are completing or already at
        // target
        if animation.is_complete() {
          false
        } else {
          // Check if target has changed significantly from the animation's
          // current target Use a reasonable threshold to avoid
          // creating animations for every tiny change
          let target_distance = (animation.target_rect.x()
            - target_rect.x())
          .abs()
            + (animation.target_rect.y() - target_rect.y()).abs()
            + (animation.target_rect.width() - target_rect.width()).abs()
            + (animation.target_rect.height() - target_rect.height())
              .abs();
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
        // Determine the start position for the new animation.
        // Cancel and replace: start from current animated position if an
        // animation is running.
        let start_rect = existing_animation
          .as_ref()
          .map_or(prev_target, WindowAnimationState::current_rect);

        let animation = WindowAnimationState::new_movement(
          start_rect,
          target_rect.clone(),
          &config.value.animations.window_move,
        );
        self.start_animation(window_id, animation);
      }

      let initial_rect = self.get_animation(&window_id).map_or_else(
        || target_rect.clone(),
        WindowAnimationState::current_rect,
      );

      let initial_opacity = self
        .get_animation(&window_id)
        .and_then(WindowAnimationState::current_opacity)
        .map(|o| o.to_f32());

      self.create_overlay(
        window_id,
        native_window_id,
        &initial_rect,
        initial_opacity,
        dispatcher,
      );

      // Start the timer after the overlay has been created.
      // TODO: Start times for animations will differ slightly between
      // windows within the same platform sync.
      if let Some(animation) = self.animations.get_mut(&window_id) {
        animation.start_time = Instant::now();
      }
    }

    if let Some(animation) = self.get_animation(&window_id) {
      let has_overlay = self.has_overlay(&window_id);

      AnimationStartResult {
        rect: if has_overlay {
          // With an overlay the real window is hidden, so move it to the
          // target immediately.
          target_rect
        } else {
          animation.current_rect()
        },
        has_overlay,
      }
    } else {
      AnimationStartResult {
        rect: target_rect,
        has_overlay: false,
      }
    }
  }

  /// Cancels any active overlay for the given window and destroys it.
  pub fn cancel_overlay(&mut self, window_id: &Uuid) {
    self.destroy_overlay(window_id);
    self.hide_surface_if_empty();
  }
}
