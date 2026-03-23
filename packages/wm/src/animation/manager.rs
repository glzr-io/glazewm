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
  animation::{engine::apply_easing, state::WindowAnimationState},
  commands::general::platform_sync,
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// Layers backing a single window animation on macOS.
#[cfg(target_os = "macos")]
enum AnimationLayers {
  /// Single layer (e.g. open animations).
  Single(wm_platform::LayerId),
  /// Two layers cross-fading during a move animation.
  Crossfade {
    from_layer: wm_platform::LayerId,
    to_layer: wm_platform::LayerId,
  },
}

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

  /// Shared animation surface with one `CALayer` per animating window.
  #[cfg(target_os = "macos")]
  surface: Option<wm_platform::AnimationSurface>,

  /// Maps window UUIDs to their `CALayer` handle(s) within the surface.
  #[cfg(target_os = "macos")]
  layer_ids: HashMap<Uuid, AnimationLayers>,

  /// Overlay windows for screenshot-based animations.
  #[cfg(target_os = "windows")]
  overlays: HashMap<Uuid, wm_platform::OverlayWindow>,
}

impl AnimationManager {
  pub fn new(animation_tick_tx: mpsc::UnboundedSender<()>) -> Self {
    Self {
      animations: HashMap::new(),
      animation_tick_tx,
      animation_timer_running: Arc::new(AtomicBool::new(false)),
      #[cfg(target_os = "macos")]
      surface: None,
      #[cfg(target_os = "macos")]
      layer_ids: HashMap::new(),
      #[cfg(target_os = "windows")]
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

  // ── Platform-specific overlay helpers
  // ──────────────────────────────────

  /// Returns whether a visual overlay exists for the given window.
  #[cfg(target_os = "macos")]
  fn has_overlay(&self, window_id: &Uuid) -> bool {
    self.layer_ids.contains_key(window_id)
  }

  /// Returns whether a visual overlay exists for the given window.
  #[cfg(target_os = "windows")]
  fn has_overlay(&self, window_id: &Uuid) -> bool {
    self.overlays.contains_key(window_id)
  }

  /// Creates or replaces the visual overlay for a window animation.
  ///
  /// For move animations, takes two screenshots (before and after the
  /// window is repositioned) and stores them as a cross-fade pair. For
  /// open animations, takes a single screenshot.
  #[cfg(target_os = "macos")]
  fn create_overlay(
    &mut self,
    window_id: Uuid,
    native_window_id: wm_platform::WindowId,
    initial_rect: &Rect,
    opacity: Option<f32>,
    is_move: bool,
    target_rect: Option<&Rect>,
    native_window: Option<&wm_platform::NativeWindow>,
    dispatcher: &wm_platform::Dispatcher,
  ) {
    // Remove any existing layer for this window.
    self.destroy_overlay(&window_id);

    // Lazy-create the shared animation surface.
    if self.surface.is_none() {
      match wm_platform::AnimationSurface::new(dispatcher) {
        Ok(surface) => self.surface = Some(surface),
        Err(err) => {
          tracing::warn!("Failed to create animation surface: {}", err);
          return;
        }
      }
    }

    let surface = self.surface.as_mut().expect("surface just created");

    // For move animations, take two screenshots and cross-fade.
    if is_move {
      if let (Some(target), Some(native_win)) =
        (target_rect, native_window)
      {
        // Screenshot A: window at its current (start) position.
        let from_layer = match surface.add_layer(
          native_window_id,
          initial_rect,
          Some(1.0),
        ) {
          Ok(id) => id,
          Err(err) => {
            tracing::warn!("Failed to add from-layer: {}", err);
            return;
          }
        };

        // Move the real window to the target so we can screenshot it
        // there.
        if let Err(err) = native_win.set_frame(target) {
          tracing::warn!(
            "Failed to move window for screenshot B: {}",
            err
          );
        }

        // Screenshot B: window at the target position.
        let to_layer = match surface.add_layer(
          native_window_id,
          initial_rect,
          Some(0.0),
        ) {
          Ok(id) => id,
          Err(err) => {
            tracing::warn!("Failed to add to-layer: {}", err);
            // Clean up the from-layer we already added.
            let _ = surface.remove_layer(from_layer);
            return;
          }
        };

        self.layer_ids.insert(
          window_id,
          AnimationLayers::Crossfade {
            from_layer,
            to_layer,
          },
        );
        return;
      }
    }

    // Single-layer path (open animations or fallback).
    match surface.add_layer(native_window_id, initial_rect, opacity) {
      Ok(layer_id) => {
        self
          .layer_ids
          .insert(window_id, AnimationLayers::Single(layer_id));
      }
      Err(err) => {
        tracing::warn!("Failed to add animation layer: {}", err);
      }
    }
  }

  /// Creates or replaces the visual overlay for a window animation.
  #[cfg(target_os = "windows")]
  fn create_overlay(
    &mut self,
    window_id: Uuid,
    native_window_id: wm_platform::WindowId,
    initial_rect: &Rect,
    opacity: Option<f32>,
    _is_move: bool,
    _target_rect: Option<&Rect>,
    _native_window: Option<&wm_platform::NativeWindow>,
    dispatcher: &wm_platform::Dispatcher,
  ) {
    // Destroy any existing overlay for this window.
    self.destroy_overlay(&window_id);

    match wm_platform::OverlayWindow::new(
      native_window_id,
      initial_rect,
      dispatcher,
    ) {
      Ok(overlay) => {
        if let Some(alpha) = opacity {
          let _ = overlay.set_opacity(alpha);
        }
        self.overlays.insert(window_id, overlay);
      }
      Err(err) => {
        tracing::warn!("Failed to create overlay window: {}", err);
      }
    }
  }

  /// Removes the visual overlay for a window.
  #[cfg(target_os = "macos")]
  fn destroy_overlay(&mut self, window_id: &Uuid) {
    if let Some(layers) = self.layer_ids.remove(window_id) {
      if let Some(surface) = &mut self.surface {
        let ids = match layers {
          AnimationLayers::Single(id) => vec![id],
          AnimationLayers::Crossfade {
            from_layer,
            to_layer,
          } => {
            vec![from_layer, to_layer]
          }
        };
        for id in ids {
          if let Err(err) = surface.remove_layer(id) {
            tracing::warn!("Failed to remove animation layer: {}", err);
          }
        }
      }
    }
  }

  /// Removes the visual overlay for a window.
  #[cfg(target_os = "windows")]
  fn destroy_overlay(&mut self, window_id: &Uuid) {
    if let Some(overlay) = self.overlays.remove(window_id) {
      if let Err(err) = overlay.destroy() {
        tracing::warn!("Failed to destroy overlay: {}", err);
      }
    }
  }

  /// Sends updated positions and opacities to the visual overlays for
  /// all in-progress animations.
  ///
  /// Opacity is derived purely from eased progress — animation-state
  /// `start_opacity`/`target_opacity` are intentionally ignored so the
  /// crossfade always works regardless of those fields.
  #[cfg(target_os = "macos")]
  fn update_overlays(&self, active_ids: &[Uuid]) {
    let mut updates: Vec<(
      wm_platform::LayerId,
      Rect,
      Option<OpacityValue>,
    )> = Vec::new();

    for id in active_ids {
      let Some(anim) = self.get_animation(id) else {
        continue;
      };
      let Some(layers) = self.layer_ids.get(id) else {
        continue;
      };

      let rect = anim.current_rect();

      match layers {
        AnimationLayers::Single(layer_id) => {
          updates.push((*layer_id, rect, anim.current_opacity()));
        }
        AnimationLayers::Crossfade {
          from_layer,
          to_layer,
        } => {
          let eased = apply_easing(anim.progress(), &anim.easing);
          let from_opacity = Some(OpacityValue(1.0 - eased));
          let to_opacity = Some(OpacityValue(eased));

          updates.push((*from_layer, rect.clone(), from_opacity));
          updates.push((*to_layer, rect, to_opacity));
        }
      }
    }

    if let Some(surface) = &self.surface {
      if let Err(err) = surface.update_layers(updates) {
        tracing::warn!("Failed to update animation layers: {}", err);
      }
    }
  }

  /// Sends updated positions and opacities to the visual overlays for
  /// all in-progress animations.
  #[cfg(target_os = "windows")]
  fn update_overlays(&self, active_ids: &[Uuid]) {
    let updates: Vec<_> = active_ids
      .iter()
      .filter_map(|id| {
        let anim = self.get_animation(id)?;
        let overlay = self.overlays.get(id)?;
        Some((overlay, anim.current_rect(), anim.current_opacity()))
      })
      .collect();

    let batch: Vec<_> = updates
      .iter()
      .map(|(overlay, rect, opacity)| (*overlay, rect, *opacity))
      .collect();

    wm_platform::move_group(&batch);
  }

  /// Tears down the shared surface when no layers remain.
  #[cfg(target_os = "macos")]
  fn destroy_surface_if_empty(&mut self) {
    let is_empty = self
      .surface
      .as_ref()
      .and_then(|s| s.has_layers().ok())
      .map_or(true, |has| !has);

    if is_empty {
      if let Some(surface) = self.surface.take() {
        if let Err(err) = surface.destroy() {
          tracing::warn!("Failed to destroy animation surface: {}", err);
        }
      }
    }
  }

  // ── Core update + animation start
  // ──────────────────────────────────

  /// Updates all active animations and redraws windows that are animating.
  #[allow(dead_code)] // Public API method, may be used externally.
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
    }

    // Destroy overlays after the real windows have been repositioned so
    // there is no visible gap.
    for window_id in &completed_ids {
      state.animation_manager.destroy_overlay(window_id);
    }

    // Tear down the shared surface when all layers are gone.
    #[cfg(target_os = "macos")]
    state.animation_manager.destroy_surface_if_empty();

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
    native_window: &wm_platform::NativeWindow,
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
    let is_move = !is_opening;
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

      // Create a visual overlay for the new animation.
      let initial_rect = self.get_animation(&window_id).map_or_else(
        || target_rect.clone(),
        WindowAnimationState::current_rect,
      );

      let initial_opacity = self
        .get_animation(&window_id)
        .and_then(|a| a.current_opacity())
        .map(|o| o.to_f32());

      self.create_overlay(
        window_id,
        native_window_id,
        &initial_rect,
        initial_opacity,
        is_move,
        Some(&target_rect),
        Some(native_window),
        dispatcher,
      );

      // Reset the start time so the animation begins after the overlay is
      // ready (overlay creation may block, e.g. for screenshot capture).
      if let Some(anim) = self.animations.get_mut(&window_id) {
        anim.start_time = std::time::Instant::now();
      }
    }

    // Get the current animation state (re-fetch after potentially starting
    // new animation)
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
    self.destroy_overlay(window_id);

    #[cfg(target_os = "macos")]
    self.destroy_surface_if_empty();
  }
}
