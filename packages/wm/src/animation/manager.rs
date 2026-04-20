use std::{
  collections::HashMap,
  sync::{atomic::AtomicBool, atomic::Ordering, Arc},
};

use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::WindowState;
use wm_platform::{NativeWindow, OpacityValue, Rect};
#[cfg(target_os = "windows")]
use wm_platform::{NativeWindowWindowsExt, ResizeSession};

use crate::{
  animation::state::{AnimationType, WindowAnimationState},
  commands::general::platform_sync,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Result of [`AnimationManager::start_animation_if_needed`], describing
/// what the caller should do with the real app window's position this frame.
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
  /// Whether the animation timer thread is currently running.
  animation_timer_running: Arc<AtomicBool>,
  /// Active resize sessions keyed by window ID.
  ///
  /// A session is created when a movement/resize animation starts with
  /// `use_surrogate = true` and is destroyed once the animation completes
  /// and the real window has been moved to its final position.
  #[cfg(target_os = "windows")]
  pub(crate) resize_sessions: HashMap<Uuid, ResizeSession>,
  /// Sessions that have been removed from `resize_sessions` after their
  /// animation completed but must outlive the final `platform_sync` call
  /// that repositions the real window.
  #[cfg(target_os = "windows")]
  pub(crate) pending_session_cleanup: Vec<ResizeSession>,
}

impl AnimationManager {
  /// Creates a new `AnimationManager`.
  pub fn new(animation_tick_tx: mpsc::UnboundedSender<()>) -> Self {
    Self {
      animations: HashMap::new(),
      animation_tick_tx,
      animation_timer_running: Arc::new(AtomicBool::new(false)),
      #[cfg(target_os = "windows")]
      resize_sessions: HashMap::new(),
      #[cfg(target_os = "windows")]
      pending_session_cleanup: Vec::new(),
    }
  }

  /// Inserts or replaces the animation state for a window.
  pub fn start_animation(
    &mut self,
    window_id: Uuid,
    animation: WindowAnimationState,
  ) {
    self.animations.insert(window_id, animation);
  }

  /// Returns the current animation state for a window, if any.
  pub fn get_animation(
    &self,
    window_id: &Uuid,
  ) -> Option<&WindowAnimationState> {
    self.animations.get(window_id)
  }

  /// Removes a window's animation and any associated resize session.
  #[allow(dead_code)]
  pub fn remove_animation(&mut self, window_id: &Uuid) {
    self.animations.remove(window_id);
    #[cfg(target_os = "windows")]
    self.resize_sessions.remove(window_id);
  }

  /// Removes all completed animations and returns their window IDs.
  ///
  /// Sessions for completed animations are moved to `pending_session_cleanup`
  /// so they remain visible until after the final `platform_sync` call has
  /// repositioned the real windows. `pre_commit` is called on each session
  /// at this point to snapshot the window's liveness and position the
  /// surrogate at the final target rect.
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
      if let Some(mut session) = self.resize_sessions.remove(id) {
        session.pre_commit();
        self.pending_session_cleanup.push(session);
      }
    }

    completed_ids
  }

  /// Whether there are any active animations.
  pub fn has_active_animations(&self) -> bool {
    !self.animations.is_empty()
  }

  /// Returns all active animation window IDs.
  pub fn active_window_ids(&self) -> Vec<Uuid> {
    self.animations.keys().copied().collect()
  }

  /// Clears all animations and any associated sessions.
  #[allow(dead_code)]
  pub fn clear(&mut self) {
    self.animations.clear();
    #[cfg(target_os = "windows")]
    {
      self.resize_sessions.clear();
      self.pending_session_cleanup.clear();
    }
  }

  /// Drains all active and pending resize sessions and returns them.
  ///
  /// Used by `WmState::Drop` to commit sessions during shutdown or crash so
  /// that no window is left at an intermediate animation position.
  #[cfg(target_os = "windows")]
  pub fn drain_all_sessions(&mut self) -> Vec<ResizeSession> {
    let mut sessions: Vec<ResizeSession> =
      self.resize_sessions.drain().map(|(_, s)| s).collect();
    sessions.extend(self.pending_session_cleanup.drain(..));
    sessions
  }

  /// Starts the animation timer if it is not already running.
  ///
  /// Fires ticks at the monitor refresh rate, capped by `max_frame_rate`.
  pub fn ensure_timer_running(
    &self,
    state: &WmState,
    config: &UserConfig,
  ) {
    if self.has_active_animations()
      && !self.animation_timer_running.load(Ordering::Relaxed) {

      self.animation_timer_running.store(true, Ordering::Relaxed);
      let tx = self.animation_tick_tx.clone();
      let timer_flag = self.animation_timer_running.clone();

      let mut frame_time_ms = 16u32;

      if let Some(container) = state.focused_container() {
        if let Some(monitor) = CommonGetters::monitor(&container) {
          let refresh_rate =
            monitor.native_properties().refresh_rate.unwrap_or(60);
          let capped_rate =
            refresh_rate.min(config.value.animations.max_frame_rate);
          frame_time_ms = 1000 / capped_rate.max(60);
        }
      } else {
        frame_time_ms =
          1000 / config.value.animations.max_frame_rate.max(60);
      }

      tokio::spawn(async move {
        let mut interval = tokio::time::interval(
          tokio::time::Duration::from_millis(frame_time_ms as u64),
        );
        interval
          .set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
          interval.tick().await;
          if !timer_flag.load(Ordering::Relaxed) || tx.send(()).is_err() {
            break;
          }
        }

        timer_flag.store(false, Ordering::Relaxed);
      });
    }
  }

  /// Updates all active animations and redraws windows that are animating.
  #[allow(dead_code)]
  pub fn update(
    &mut self,
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    Self::update_internal(state, config)
  }

  /// Internal update, accessed through `WmState` to avoid double-borrow.
  pub(crate) fn update_internal(
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    if !state.animation_manager.has_active_animations() {
      return Ok(());
    }

    // Queue in-progress windows for redraw, skipping floating windows (they
    // are never animated).
    let active_window_ids: Vec<_> = state
      .animation_manager
      .active_window_ids()
      .into_iter()
      .filter(|id| {
        if let Some(container) = state.container_by_id(*id) {
          if let Ok(window) = container.as_window_container() {
            if matches!(window.state(), WindowState::Floating(_)) {
              return false;
            }
          }
        }
        state
          .animation_manager
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

    // Remove completed animations. Their sessions are moved to
    // `pending_session_cleanup` and must outlive the `platform_sync` call
    // below so the real window is repositioned before surrogates disappear.
    let completed_ids =
      state.animation_manager.remove_completed_animations();

    // Queue completed animations for a final redraw so `platform_sync` moves
    // the real window to its target position.
    for window_id in &completed_ids {
      if let Some(container) = state.container_by_id(*window_id) {
        if let Ok(window) = container.as_window_container() {
          state.pending_sync.queue_container_to_redraw(window);
        }
      }
    }

    if state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    // Clear pending sessions now that `platform_sync` has moved the real
    // windows to their final positions. Dropping each session destroys its
    // surrogate overlay.
    #[cfg(target_os = "windows")]
    state.animation_manager.pending_session_cleanup.clear();

    // Keep the timer running while animations are active; stop it otherwise
    // so the background thread exits cleanly.
    if state.animation_manager.has_active_animations() {
      state.animation_manager.ensure_timer_running(state, config);
    } else {
      state
        .animation_manager
        .animation_timer_running
        .store(false, Ordering::Relaxed);
    }

    Ok(())
  }

  /// Determines whether a new animation should be started for a window.
  fn should_start_new_animation(
    &self,
    window_id: &Uuid,
    is_opening: bool,
    is_resize: bool,
    target_rect: &Rect,
    previous_target: Option<&Rect>,
    config: &UserConfig,
  ) -> bool {
    let existing_animation = self.get_animation(window_id);

    let anim_config = if is_resize {
      &config.value.animations.window_resize
    } else {
      &config.value.animations.window_move
    };
    let threshold = anim_config.threshold_px as i32;

    if is_opening && config.value.animations.window_open.enabled {
      existing_animation.is_none()
    } else if !is_opening && anim_config.enabled {
      if let Some(anim) = existing_animation {
        if anim.is_complete() {
          false
        } else {
          // Check whether the target has changed enough from the current
          // animation target to warrant a cancel-and-replace.
          let target_distance =
            (anim.target_rect.x() - target_rect.x()).abs()
              + (anim.target_rect.y() - target_rect.y()).abs()
              + (anim.target_rect.width() - target_rect.width()).abs()
              + (anim.target_rect.height() - target_rect.height()).abs();
          target_distance > threshold
        }
      } else if let Some(prev_target) = previous_target {
        let distance = (prev_target.x() - target_rect.x()).abs()
          + (prev_target.y() - target_rect.y()).abs()
          + (prev_target.width() - target_rect.width()).abs()
          + (prev_target.height() - target_rect.height()).abs();
        distance > threshold
      } else {
        false
      }
    } else {
      false
    }
  }

  /// Determines the rect and opacity to use for a window this frame.
  ///
  /// Starts a new animation when movement or resize crosses the configured
  /// threshold. Returns [`AnimationPositionResult::Frozen`] while a
  /// surrogate overlay is active so the caller does not reposition the real
  /// window on intermediate frames.
  pub fn start_animation_if_needed(
    &mut self,
    window_id: Uuid,
    is_opening: bool,
    is_resize: bool,
    target_rect: Rect,
    previous_target: Option<Rect>,
    // Only used on Windows to capture the window for the surrogate.
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    native_window: &NativeWindow,
    config: &UserConfig,
  ) -> (AnimationPositionResult, Option<OpacityValue>) {
    let existing_animation = self.get_animation(&window_id).cloned();

    let should_start = self.should_start_new_animation(
      &window_id,
      is_opening,
      is_resize,
      &target_rect,
      previous_target.as_ref(),
      config,
    );

    if should_start {
      if is_opening {
        let animation = WindowAnimationState::new_open(
          target_rect.clone(),
          &config.value.animations.window_open,
        );
        self.start_animation(window_id, animation);

        // Create a surrogate for the open animation so the window fades in
        // via DWM thumbnail opacity rather than per-frame `SetWindowPos`.
        // Both source and target are `target_rect` — the overlay sits still
        // while only the opacity animates.
        #[cfg(target_os = "windows")]
        if !self.resize_sessions.contains_key(&window_id) {
          match ResizeSession::begin(
            native_window.hwnd(),
            &target_rect,
            &target_rect,
            None,
          ) {
            Ok(session) => {
              self.resize_sessions.insert(window_id, session);
            }
            Err(err) => {
              tracing::warn!(
                "Failed to begin open surrogate for window {window_id}: \
                 {err}."
              );
            }
          }
        }
      } else if let Some(prev_target) = previous_target {
        // Start from the current animated position on cancel-and-replace so
        // the animation does not jump back to the original start.
        let start_rect = existing_animation
          .as_ref()
          .map(|a| a.current_rect())
          .unwrap_or_else(|| prev_target.clone());

        let anim_config = if is_resize {
          &config.value.animations.window_resize
        } else {
          &config.value.animations.window_move
        };

        let animation = WindowAnimationState::new_movement(
          start_rect.clone(),
          target_rect.clone(),
          anim_config,
        );
        self.start_animation(window_id, animation);

        // Redirect an in-flight surrogate session to the new target, or
        // create a new one. For pure translations (no size change) skip
        // creation — direct frame-by-frame repositioning is smooth and avoids
        // the capture overhead. Also skip when the cancelled animation was an
        // `Open` whose surrogate failed to create, to avoid cloaking the
        // window without a valid overlay.
        #[cfg(target_os = "windows")]
        let has_size_change = start_rect.width() != target_rect.width()
          || start_rect.height() != target_rect.height();
        #[cfg(target_os = "windows")]
        let is_replacing_open = existing_animation
          .as_ref()
          .map(|a| matches!(a.animation_type, AnimationType::Open))
          .unwrap_or(false);
        // Redirect an in-flight surrogate session to the new target, or
        // create a new one when none is active. Redirection keeps the
        // existing surrogate alive so the real window is never left
        // uncloaked at an intermediate position.
        #[cfg(target_os = "windows")]
        if let Some(session) = self.resize_sessions.get_mut(&window_id) {
          session.update_target(&target_rect);
        } else if has_size_change && !is_replacing_open
        {
          match ResizeSession::begin(
            native_window.hwnd(),
            &start_rect,
            &target_rect,
            anim_config.surrogate_color.as_ref(),
          ) {
            Ok(session) => {
              self.resize_sessions.insert(window_id, session);
            }
            Err(err) => {
              tracing::warn!(
                "Failed to begin resize session for window {window_id}: \
                 {err}."
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

      // If a surrogate overlay is active, update it and tell the caller to
      // leave the real window untouched this frame. Only freeze when the
      // surrogate was successfully created — if creation failed the session
      // exists but `has_surrogate()` is false, and we fall through to `Apply`
      // so the real window is animated normally instead of disappearing.
      #[cfg(target_os = "windows")]
      if let Some(session) = self.resize_sessions.get_mut(&window_id) {
        let opacity_u8 =
          opacity.as_ref().map(|o| o.to_alpha()).unwrap_or(255);
        session.update(&current_rect, opacity_u8);
        if session.has_surrogate() {
          return (AnimationPositionResult::Frozen, None);
        }
      }

      (AnimationPositionResult::Apply(current_rect), opacity)
    } else {
      // No animation in the map — either the animation completed and
      // `remove_completed_animations` was already called, or animations are
      // disabled. Apply the final target rect directly.
      (AnimationPositionResult::Apply(target_rect), None)
    }
  }
}

