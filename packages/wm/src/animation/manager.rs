use std::{
  collections::HashMap,
  sync::{atomic::AtomicBool, atomic::Ordering, Arc},
};

use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::WindowState;
use wm_platform::{NativeWindow, OpacityValue, Rect};
#[cfg(target_os = "windows")]
use wm_platform::{NativeWindowWindowsExt, ResizeSession, WorkspaceSurrogate};

use crate::{
  animation::state::WindowAnimationState,
  commands::general::platform_sync,
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
  wm_state::WmState,
};

/// Tracks a single window's participation in the current workspace-switch
/// slide animation.
#[cfg(target_os = "windows")]
struct WorkspaceSwitchEntry {
  /// Surrogate overlay that slides across the monitor each frame.
  surrogate: Option<WorkspaceSurrogate>,
  /// `true` for windows on the incoming workspace, `false` for outgoing.
  is_incoming: bool,
}

/// Shared state for all windows in a workspace-switch slide animation.
///
/// A single elapsed-time driver advances all surrogates in lock-step so every
/// window translates by the same pixel offset on every frame, preserving the
/// illusion that both workspaces move as a single connected panel.
#[cfg(target_os = "windows")]
struct WorkspaceSwitchState {
  /// All participating windows keyed by window ID.
  windows: HashMap<Uuid, WorkspaceSwitchEntry>,
  /// Progress driver. Only `progress()` and `easing` are used.
  driver: WindowAnimationState,
  /// Slide direction: `+1` = target workspace is higher-index (incoming from
  /// right, outgoing to left). `-1` = opposite. `0` = fade in place.
  direction: i32,
  /// Left x-coordinate of the animation monitor in screen pixels.
  monitor_x: i32,
  /// Width of the animation monitor in screen pixels.
  monitor_width: i32,
}

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
  /// Active workspace-switch slide animation, or `None` when idle.
  #[cfg(target_os = "windows")]
  workspace_switch: Option<WorkspaceSwitchState>,
  /// Workspace-switch state that just completed; kept alive until the final
  /// `platform_sync` call unclocks the incoming real windows.
  #[cfg(target_os = "windows")]
  pending_ws_cleanup: Option<WorkspaceSwitchState>,
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
      #[cfg(target_os = "windows")]
      workspace_switch: None,
      #[cfg(target_os = "windows")]
      pending_ws_cleanup: None,
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

  /// Whether there are any active animations or a workspace-switch in flight.
  pub fn has_active_animations(&self) -> bool {
    if !self.animations.is_empty() {
      return true;
    }
    #[cfg(target_os = "windows")]
    if self.workspace_switch.is_some() {
      return true;
    }
    false
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
      self.workspace_switch = None;
      self.pending_ws_cleanup = None;
    }
  }

  /// Drains all active and pending resize sessions and returns them.
  ///
  /// Used by `WmState::Drop` to commit sessions during shutdown or crash so
  /// that no window is left at an intermediate animation position. Workspace-
  /// switch surrogates are also dropped (real windows are already at their
  /// final positions by the time this is called).
  #[cfg(target_os = "windows")]
  pub fn drain_all_sessions(&mut self) -> Vec<ResizeSession> {
    let mut sessions: Vec<ResizeSession> =
      self.resize_sessions.drain().map(|(_, s)| s).collect();
    sessions.extend(self.pending_session_cleanup.drain(..));
    self.workspace_switch = None;
    self.pending_ws_cleanup = None;
    sessions
  }

  /// Starts the animation timer if it is not already running.
  ///
  /// Fires ticks aligned to DWM composition frames via `DwmFlush`, ensuring
  /// surrogate updates reach the compositor on every rendered frame without
  /// timer-resolution jitter. Ticks are capped to `max_frame_rate` by
  /// inserting a minimum inter-frame sleep when the monitor refresh rate
  /// exceeds the configured cap.
  pub fn ensure_timer_running(
    &self,
    _state: &WmState,
    config: &UserConfig,
  ) {
    if self.has_active_animations()
      && !self.animation_timer_running.load(Ordering::Relaxed) {

      self.animation_timer_running.store(true, Ordering::Relaxed);
      let tx = self.animation_tick_tx.clone();
      let timer_flag = self.animation_timer_running.clone();

      // Compute the minimum inter-frame interval in microseconds from the
      // capped frame rate. `DwmFlush` waits for the next composition frame
      // naturally; the sleep is only needed when the monitor refresh rate
      // exceeds `max_frame_rate`.
      let max_rate = config.value.animations.max_frame_rate.max(1);
      let min_frame_us = 1_000_000u64 / max_rate as u64;

      // Spawn a real OS thread (not a Tokio task) so it can call the
      // blocking `DwmFlush` without stalling the async runtime.
      let timer_flag_err = timer_flag.clone();
      std::thread::Builder::new()
        .name("glazewm-anim-tick".into())
        .spawn(move || {
          let mut last_tick = std::time::Instant::now();

          loop {
            if !timer_flag.load(Ordering::Relaxed) {
              break;
            }

            // Wait for the next DWM composition frame. On non-Windows
            // builds this is a no-op inside `dwm_flush`, so we fall back
            // to a plain sleep to avoid a busy-loop.
            wm_platform::dwm_flush();
            #[cfg(not(target_os = "windows"))]
            std::thread::sleep(std::time::Duration::from_micros(
              min_frame_us,
            ));

            // Enforce the max frame rate cap: skip this tick if we haven't
            // yet reached the minimum inter-frame interval.
            let now = std::time::Instant::now();
            let elapsed_us = now.duration_since(last_tick).as_micros() as u64;
            if elapsed_us < min_frame_us {
              continue;
            }
            last_tick = now;

            if tx.send(()).is_err() {
              break;
            }
          }

          timer_flag.store(false, Ordering::Relaxed);
        })
        .unwrap_or_else(|err| {
          tracing::warn!("Failed to spawn animation tick thread: {err}.");
          timer_flag_err.store(false, Ordering::Relaxed);
          std::thread::spawn(|| {})
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

    // Drive workspace-switch slide surrogates. All windows share a single
    // elapsed-time driver so every surrogate translates by the same pixel
    // offset each frame, making both workspaces move as one connected panel.
    //
    // This runs before `platform_sync` so that when the animation completes,
    // the incoming windows are queued for redraw and uncloaked in the same
    // tick.
    #[cfg(target_os = "windows")]
    let ws_complete_ids: Option<Vec<Uuid>> = {
      use crate::animation::engine::{animation_progress, apply_easing};

      if let Some(ws) = &mut state.animation_manager.workspace_switch {
        let raw_progress =
          animation_progress(ws.driver.start_time, ws.driver.duration);
        let eased = apply_easing(raw_progress, &ws.driver.easing);

        for entry in ws.windows.values_mut() {
          if let Some(ref mut s) = entry.surrogate {
            s.update_slide(
              eased,
              entry.is_incoming,
              ws.direction,
              ws.monitor_x,
              ws.monitor_width,
            );
          }
        }

        if raw_progress >= 1.0 {
          Some(ws.windows.keys().copied().collect())
        } else {
          None
        }
      } else {
        None
      }
    };

    // On completion, move surrogates to pending cleanup so they outlive the
    // final `platform_sync` call that unclocks the incoming real windows.
    #[cfg(target_os = "windows")]
    if let Some(ids) = ws_complete_ids {
      state.animation_manager.pending_ws_cleanup =
        state.animation_manager.workspace_switch.take();

      for id in ids {
        if let Some(container) = state.container_by_id(id) {
          if let Ok(window) = container.as_window_container() {
            state.pending_sync.queue_container_to_redraw(window);
          }
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
    {
      state.animation_manager.pending_session_cleanup.clear();
      // Flush before dropping workspace-switch surrogates. DWM thumbnails do
      // not capture the window's compositor shadow, so shadows are absent
      // while windows are cloaked and appear suddenly on uncloak. One flush
      // after `platform_sync` (which unclocks real windows) lets DWM render
      // one frame with the real windows — including their shadows — while the
      // surrogates are still live. Dropping surrogates after that frame makes
      // the transition seamless.
      if state.animation_manager.pending_ws_cleanup.is_some() {
        wm_platform::dwm_flush();
      }
      state.animation_manager.pending_ws_cleanup = None;
    }

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

    if anim_config.enabled {
      if let Some(anim) = existing_animation {
        if anim.is_complete() {
          // Animation already at its target — treat as a static window and
          // apply the threshold check against the completed target so a new
          // animation starts if the window needs to move.
          let distance = (anim.target_rect.x() - target_rect.x()).abs()
            + (anim.target_rect.y() - target_rect.y()).abs()
            + (anim.target_rect.width() - target_rect.width()).abs()
            + (anim.target_rect.height() - target_rect.height()).abs();
          distance > threshold
        } else {
          // Redirect any in-progress animation to the new target whenever the
          // destination changes, regardless of distance. Without this, small
          // target adjustments (< threshold) are silently swallowed and the
          // window snaps after the stale animation finishes.
          anim.target_rect != *target_rect
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
  /// threshold.
  ///
  /// Returns [`AnimationPositionResult::Frozen`] while a surrogate overlay
  /// is active so the caller does not reposition the real window on
  /// intermediate frames.
  pub fn start_animation_if_needed(
    &mut self,
    window_id: Uuid,
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
      is_resize,
      &target_rect,
      previous_target.as_ref(),
      config,
    );

    if should_start {
      if let Some(prev_target) = previous_target {
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
        // create a new one. The surrogate overlay is our own window and moves
        // instantly each frame; the real window only needs one async move to
        // its final position. This avoids per-frame cross-process
        // `SWP_ASYNCWINDOWPOS` calls, which lag behind when the target
        // process's message loop is slow.
        #[cfg(target_os = "windows")]
        if let Some(session) = self.resize_sessions.get_mut(&window_id) {
          session.update_target(&target_rect);
        } else {
          match ResizeSession::begin(
            native_window.hwnd(),
            &start_rect,
            &target_rect,
            anim_config.surrogate_color.as_ref(),
            false,
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

  /// Returns `true` while `window_id` is an incoming participant in the
  /// active workspace-switch animation.
  ///
  /// Unlike the `pending_sync` incoming flag (cleared after the first
  /// `platform_sync`), this stays `true` for the full animation duration so
  /// that focus events during the animation do not prematurely uncloak the
  /// real window before the surrogate finishes sliding in.
  #[cfg(target_os = "windows")]
  pub fn is_workspace_switch_incoming(&self, window_id: &Uuid) -> bool {
    self
      .workspace_switch
      .as_ref()
      .and_then(|ws| ws.windows.get(window_id))
      .map(|e| e.is_incoming)
      .unwrap_or(false)
  }

  /// Installs a workspace-switch slide animation for the provided windows.
  ///
  /// Accepts pre-created [`WorkspaceSurrogate`] instances together with their
  /// incoming/outgoing flags. A shared driver advances all surrogates in
  /// lock-step so the entire workspace moves as one panel. Any previous
  /// workspace-switch state is dropped.
  #[cfg(target_os = "windows")]
  pub fn start_workspace_switch(
    &mut self,
    windows: Vec<(Uuid, Option<WorkspaceSurrogate>, bool)>,
    direction: i32,
    monitor_x: i32,
    monitor_width: i32,
    config: &UserConfig,
  ) {
    self.workspace_switch = None;

    let ws_config = &config.value.animations.workspace_switch;
    let mut anim_config = ws_config.as_anim_type_config();

    // Scale duration proportionally to monitor width so animation speed
    // (px/s) stays constant across different screen widths. 1920 px is the
    // reference: narrower monitors keep the configured duration, wider ones
    // scale up so per-frame pixel steps remain equally smooth.
    const REFERENCE_WIDTH_PX: f32 = 1920.0;
    anim_config.duration_ms = (anim_config.duration_ms as f32
      * (monitor_width as f32 / REFERENCE_WIDTH_PX).max(1.0))
      .round() as u32;

    let dummy = Rect::from_xy(0, 0, 1, 1);
    let driver =
      WindowAnimationState::new_movement(dummy.clone(), dummy, &anim_config);

    let ws_windows: HashMap<Uuid, WorkspaceSwitchEntry> = windows
      .into_iter()
      .map(|(id, surrogate, is_incoming)| {
        (id, WorkspaceSwitchEntry { surrogate, is_incoming })
      })
      .collect();

    if !ws_windows.is_empty() {
      tracing::info!(
        "Starting workspace-switch slide: direction={}, monitor_x={}, \
         monitor_width={}, windows={}",
        direction,
        monitor_x,
        monitor_width,
        ws_windows.len(),
      );
      self.workspace_switch = Some(WorkspaceSwitchState {
        windows: ws_windows,
        driver,
        direction,
        monitor_x,
        monitor_width,
      });
    } else {
      tracing::warn!("Workspace-switch skipped: no windows to animate.");
    }
  }
}

