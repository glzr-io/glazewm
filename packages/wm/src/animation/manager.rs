use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  time::{Duration, Instant},
};

use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::{EasingFunction, WindowTransitionStyle, WorkspaceSwitchStyle};
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

/// Shared state for all windows in a workspace-switch animation.
///
/// A single elapsed-time driver advances all surrogates in lock-step so every
/// window translates by the same pixel offset on every frame, preserving the
/// illusion that both workspaces move as a single connected panel.
#[cfg(target_os = "windows")]
struct WorkspaceSwitchState {
  /// All participating windows keyed by window ID.
  windows: HashMap<Uuid, WorkspaceSwitchEntry>,
  /// Time of the first rendered frame, lazily set on the first tick.
  ///
  /// Initialized to `None` so the clock starts when `update_internal` first
  /// renders the animation rather than when `start_workspace_switch` is called
  /// mid-`platform_sync`. Without lazy init, a cold-start gap of 1-3 DWM
  /// frames causes surrogates to jump ahead on their first visible tick.
  start_time: Option<Instant>,
  /// Total animation duration.
  duration: Duration,
  /// Easing function applied to raw elapsed-time progress.
  easing: EasingFunction,
  /// Transition style (slide horizontal/vertical or fade).
  style: WorkspaceSwitchStyle,
  /// Slide direction: `+1` = target workspace is higher-index (incoming from
  /// the far edge, outgoing to the near edge). `-1` = opposite.
  direction: i32,
  /// Left x-coordinate of the animation monitor in screen pixels.
  monitor_x: i32,
  /// Width of the animation monitor in screen pixels.
  monitor_width: i32,
  /// Top y-coordinate of the animation monitor in screen pixels.
  monitor_y: i32,
  /// Height of the animation monitor in screen pixels.
  monitor_height: i32,
}

/// Result of [`AnimationManager::start_animation_if_needed`], describing
/// what the caller should do with the real app window's position this frame.
pub enum AnimationPositionResult {
  /// Apply this rect to the real window via `reposition_window`.
  ///
  /// The carried `Rect` is the current animated position, available for
  /// callers that bypass the surrogate path (e.g. future macOS support).
  #[allow(dead_code)]
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
  /// Monitor rects for active slide-in (window-open) animations, keyed by
  /// window ID. Used to hide the surrogate while it is fully off the monitor.
  #[cfg(target_os = "windows")]
  slide_in_monitor_rects: HashMap<Uuid, Rect>,
  /// Active workspace-switch slide animation, or `None` when idle.
  #[cfg(target_os = "windows")]
  workspace_switch: Option<WorkspaceSwitchState>,
  /// Workspace-switch state that just completed; kept alive until the final
  /// `platform_sync` call unclocks the incoming real windows.
  #[cfg(target_os = "windows")]
  pending_ws_cleanup: Option<WorkspaceSwitchState>,
  /// Windows with an active close animation, keyed by window ID.
  ///
  /// The stored value is the raw `HWND` (as `isize`) so `WM_CLOSE` can be
  /// sent after the fade finishes without borrowing the window container.
  #[cfg(target_os = "windows")]
  pending_close_windows: HashMap<Uuid, isize>,
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
      slide_in_monitor_rects: HashMap::new(),
      #[cfg(target_os = "windows")]
      workspace_switch: None,
      #[cfg(target_os = "windows")]
      pending_ws_cleanup: None,
      #[cfg(target_os = "windows")]
      pending_close_windows: HashMap::new(),
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
  pub fn remove_animation(&mut self, window_id: &Uuid) {
    self.animations.remove(window_id);
    #[cfg(target_os = "windows")]
    self.resize_sessions.remove(window_id);
    #[cfg(target_os = "windows")]
    self.slide_in_monitor_rects.remove(window_id);
    #[cfg(target_os = "windows")]
    self.pending_close_windows.remove(window_id);
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
      #[cfg(target_os = "windows")]
      self.slide_in_monitor_rects.remove(id);
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
    // On WM shutdown close-animation windows are left open — only clear
    // the tracking state without sending WM_CLOSE.
    self.pending_close_windows.clear();
    sessions
  }

  /// Starts the animation timer if it is not already running.
  ///
  /// On Windows, ticks are aligned to DWM composition frames via `DwmFlush`,
  /// which naturally caps the rate to the monitor's refresh rate. On
  /// non-Windows, `DwmFlush` is a no-op, so a fixed 60 fps sleep is used.
  pub fn ensure_timer_running(&self) {
    if self.has_active_animations()
      && !self.animation_timer_running.load(Ordering::Relaxed) {

      self.animation_timer_running.store(true, Ordering::Relaxed);
      let tx = self.animation_tick_tx.clone();
      let timer_flag = self.animation_timer_running.clone();

      // Spawn a real OS thread (not a Tokio task) so it can call the
      // blocking `DwmFlush` without stalling the async runtime.
      let timer_flag_err = timer_flag.clone();
      std::thread::Builder::new()
        .name("glazewm-anim-tick".into())
        .spawn(move || {
          // Elevate priority so scheduling jitter between the DwmFlush
          // VSync wake-up and tick delivery is minimised.
          wm_platform::set_thread_priority_highest();

          // Send an immediate tick so the first animation frame begins
          // without waiting for the next VSync. Without this the surrogate
          // is frozen at its start position for up to one full frame period
          // (~16 ms at 60 Hz, ~8 ms at 120 Hz) before any movement begins.
          if tx.send(()).is_err() {
            timer_flag.store(false, Ordering::Relaxed);
            return;
          }

          loop {
            if !timer_flag.load(Ordering::Relaxed) {
              break;
            }

            // On Windows, `DwmFlush` blocks until the next VSync, which
            // naturally limits ticks to the monitor's refresh rate. On
            // non-Windows it is a no-op, so fall back to a 60 fps sleep.
            wm_platform::dwm_flush();
            #[cfg(not(target_os = "windows"))]
            std::thread::sleep(std::time::Duration::from_micros(16_667));

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

  /// Internal update, accessed through `WmState` to avoid double-borrow.
  pub(crate) fn update_internal(
    state: &mut WmState,
    config: &UserConfig,
  ) -> anyhow::Result<()> {
    if !state.animation_manager.has_active_animations() {
      return Ok(());
    }

    // Queue in-progress windows for redraw.
    let active_window_ids: Vec<_> = state
      .animation_manager
      .active_window_ids()
      .into_iter()
      .filter(|id| {
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

    // Finalize completed close animations before `remove_completed_animations`
    // so that their sessions are dropped directly (not moved to
    // `pending_session_cleanup`) and the normal `Apply` branch in
    // `platform_sync` never uncloaks these windows.
    #[cfg(target_os = "windows")]
    {
      use crate::commands::window::unmanage_window;

      let close_done: Vec<Uuid> = state
        .animation_manager
        .pending_close_windows
        .keys()
        .filter(|id| {
          state
            .animation_manager
            .get_animation(id)
            .map_or(false, |a| a.is_complete())
        })
        .copied()
        .collect();

      for id in close_done {
        // Drop surrogate and animation directly — bypasses pending_session_cleanup
        // so platform_sync does not attempt to reposition and uncloak the window.
        state.animation_manager.animations.remove(&id);
        state.animation_manager.resize_sessions.remove(&id);
        state.animation_manager.pending_close_windows.remove(&id);

        if let Some(container) = state.container_by_id(id) {
          if let Ok(window) = container.as_window_container() {
            if let Err(err) = window.native().close() {
              tracing::warn!("Failed to send WM_CLOSE for {id}: {err}.");
            }
            unmanage_window(window, state)?;
          }
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
        let start = *ws.start_time.get_or_insert_with(Instant::now);
        let raw_progress = animation_progress(start, ws.duration);
        let eased = apply_easing(raw_progress, &ws.easing);

        // Complete early once eased progress reaches 99% for non-overshooting
        // curves — decelerating easing spends the final ~22% of wall time
        // covering the last 1% of distance, which looks "stuck" at the
        // destination. Overshooting curves always run to full wall-clock
        // duration to preserve their bounce.
        let ws_done = if ws.easing.can_overshoot() {
          raw_progress >= 1.0
        } else {
          raw_progress >= 1.0 || eased >= 0.99
        };

        // When completing early (eased < 1.0), snap surrogates to 1.0 so
        // they sit exactly at the final window position. Without this, a
        // ~1% gap between surrogate and the just-uncloaked real window
        // exposes the desktop for one frame.
        let eased_final = if ws_done { 1.0 } else { eased };

        for entry in ws.windows.values_mut() {
          if let Some(ref mut s) = entry.surrogate {
            match ws.style {
              WorkspaceSwitchStyle::SlideHorizontal
              | WorkspaceSwitchStyle::SlideCrossfadeHorizontal
              | WorkspaceSwitchStyle::SlideFadeOutHorizontal
              | WorkspaceSwitchStyle::SlideFadeInHorizontal => {
                s.update_slide_horizontal(
                  eased_final,
                  entry.is_incoming,
                  ws.direction,
                  ws.monitor_x,
                  ws.monitor_width,
                );
              }
              WorkspaceSwitchStyle::SlideVertical
              | WorkspaceSwitchStyle::SlideCrossfadeVertical
              | WorkspaceSwitchStyle::SlideFadeOutVertical
              | WorkspaceSwitchStyle::SlideFadeInVertical => {
                s.update_slide_vertical(
                  eased_final,
                  entry.is_incoming,
                  ws.direction,
                  ws.monitor_y,
                  ws.monitor_height,
                );
              }
              WorkspaceSwitchStyle::Fade => {
                s.update_fade(eased_final, entry.is_incoming);
              }
              WorkspaceSwitchStyle::Zoom => {
                s.update_zoom(eased_final, entry.is_incoming);
              }
            }
          }
        }

        if ws_done {
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

      // Re-queue focus: `sync_focus` suppressed `SetForegroundWindow` while
      // the animation was running to prevent the OS from asynchronously
      // uncloaking the incoming focused window mid-slide. Now that the
      // surrogates are done and incoming windows are about to be uncloaked,
      // it is safe to transfer OS focus.
      state.pending_sync.queue_focus_change();
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
      // after `platform_sync` (which uncloaks real windows) lets DWM render
      // one frame with the real windows — including their shadows — while the
      // surrogates are still live. Per-window thumbnail hides were already
      // issued inside `platform_sync` immediately after each `set_cloaked(false)`
      // call, so this flush shows only real windows without double-blend.
      if state.animation_manager.pending_ws_cleanup.is_some() {
        wm_platform::dwm_flush();
      }
      state.animation_manager.pending_ws_cleanup = None;
    }

    // Keep the timer running while animations are active; stop it otherwise
    // so the background thread exits cleanly.
    if state.animation_manager.has_active_animations() {
      state.animation_manager.ensure_timer_running();
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

    let (enabled, threshold_px) = if is_resize {
      let c = &config.value.animations.window_resize;
      (c.enabled, c.threshold_px)
    } else {
      let c = &config.value.animations.window_move;
      (c.enabled, c.threshold_px)
    };
    let threshold = threshold_px as i32;

    if enabled {
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
    // Opacity from window-effects config; used as surrogate opacity when the
    // animation has no per-frame fade component.
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    effect_opacity: u8,
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

        let (duration_ms, easing) = if is_resize {
          let c = &config.value.animations.window_resize;
          (c.duration_ms, c.easing.clone())
        } else {
          let c = &config.value.animations.window_move;
          (c.duration_ms, c.easing.clone())
        };

        let animation = WindowAnimationState::new_movement(
          start_rect.clone(),
          target_rect.clone(),
          duration_ms,
          easing,
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
          let surrogate_color = if is_resize {
            config.value.animations.window_resize.surrogate_color.as_ref()
          } else {
            None
          };
          match ResizeSession::begin(
            native_window.hwnd(),
            &start_rect,
            &target_rect,
            surrogate_color,
            effect_opacity,
            true,
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
      let (current_rect, opacity) = animation.current_state();

      // Drive the surrogate overlay when one is active. `has_surrogate()`
      // requires a valid DWM thumbnail — if thumbnail registration failed (e.g.
      // elevated/UWP window), the surrogate is transparent and useless: snap
      // the window to target and clean up rather than cloaking it behind an
      // empty overlay.
      // Extract the session status with a shared borrow first, then take a
      // mutable borrow only for the drive path. Avoids a triple-lookup
      // (contains_key → get → get_mut) on the same key.
      #[cfg(target_os = "windows")]
      let session_status = self
        .resize_sessions
        .get(&window_id)
        .map(|s| (s.has_surrogate(), s.effect_opacity, s.zoom));

      #[cfg(target_os = "windows")]
      match session_status {
        Some((true, effect_opacity, zoom)) => {
          let monitor_rect =
            self.slide_in_monitor_rects.get(&window_id).cloned();
          let session =
            self.resize_sessions.get_mut(&window_id).unwrap();
          let opacity_u8 = opacity
            .as_ref()
            .map(|o| o.to_alpha())
            .unwrap_or(effect_opacity);
          if zoom {
            // Extract progress with a separate borrow before mutably using session.
            let progress = self
              .animations
              .get(&window_id)
              .map(|a| a.eased_progress())
              .unwrap_or(1.0);
            let is_close =
              self.pending_close_windows.contains_key(&window_id);
            let forward_progress =
              if is_close { 1.0 - progress } else { progress };
            let session = self.resize_sessions.get_mut(&window_id).unwrap();
            session.update_zoom_fade(forward_progress, opacity_u8);
          } else if let Some(monitor_rect) = monitor_rect {
            session.update_clipped(&current_rect, &monitor_rect, opacity_u8);
          } else {
            session.update(&current_rect, opacity_u8);
          }
          return (AnimationPositionResult::Frozen, None);
        }
        Some((false, _, _)) => {
          // Thumbnail failed — drop the transparent surrogate and snap.
          self.resize_sessions.remove(&window_id);
          self.animations.remove(&window_id);
          return (AnimationPositionResult::Apply(target_rect), None);
        }
        None => {}
      }

      (AnimationPositionResult::Apply(current_rect), opacity)
    } else {
      // No animation in the map — either the animation completed and
      // `remove_completed_animations` was already called, or animations are
      // disabled. Apply the final target rect directly.
      (AnimationPositionResult::Apply(target_rect), None)
    }
  }

  /// Returns `true` while a workspace-switch slide animation is in progress
  /// or its surrogates are still live during post-animation cleanup.
  ///
  /// Includes `pending_ws_cleanup` so that callers (e.g. tab-bar visibility,
  /// focus deferral) stay in their animation-active state until surrogates
  /// are fully dropped, preventing a one-frame flash between animation
  /// completion and surrogate teardown.
  #[cfg(target_os = "windows")]
  pub fn is_workspace_switch_active(&self) -> bool {
    self.workspace_switch.is_some() || self.pending_ws_cleanup.is_some()
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

  /// Installs a workspace-switch animation for the provided windows.
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
    monitor_y: i32,
    monitor_height: i32,
    config: &UserConfig,
  ) {
    self.workspace_switch = None;

    let ws_config = &config.value.animations.workspace_switch;

    let duration_ms = ws_config.duration_ms;

    let ws_windows: HashMap<Uuid, WorkspaceSwitchEntry> = windows
      .into_iter()
      .map(|(id, surrogate, is_incoming)| {
        (id, WorkspaceSwitchEntry { surrogate, is_incoming })
      })
      .collect();

    if !ws_windows.is_empty() {
      tracing::info!(
        "Starting workspace-switch animation: style={:?}, direction={}, \
         monitor=({monitor_x},{monitor_y},{monitor_width}x{monitor_height}), \
         windows={}",
        ws_config.style,
        direction,
        ws_windows.len(),
      );
      self.workspace_switch = Some(WorkspaceSwitchState {
        windows: ws_windows,
        start_time: None,
        duration: Duration::from_millis(u64::from(duration_ms)),
        easing: ws_config.easing.clone(),
        style: ws_config.style.clone(),
        direction,
        monitor_x,
        monitor_width,
        monitor_y,
        monitor_height,
      });
    } else {
      tracing::warn!("Workspace-switch skipped: no windows to animate.");
    }
  }

  /// Starts an open animation for a newly appearing window.
  ///
  /// The surrogate animates from a computed start state (determined by
  /// `window_open.direction`) to the window's final target rect. A
  /// `ResizeSession` handles all visuals; the real window remains cloaked
  /// until the animation completes.
  ///
  /// No-ops when `direction` is `Fade` and `opacity_from` is `1.0` (nothing
  /// would visually change for the duration).
  #[cfg(target_os = "windows")]
  pub fn start_open_animation(
    &mut self,
    window_id: Uuid,
    target_rect: Rect,
    monitor_rect: Rect,
    effect_opacity: u8,
    config: &UserConfig,
    native_window: &NativeWindow,
  ) {
    let anim_config = &config.value.animations.window_open;
    let is_zoom = anim_config.style == WindowTransitionStyle::Zoom;
    let is_stationary = anim_config.style.is_stationary();

    // Skip when there is nothing to animate.
    if is_stationary && !is_zoom && anim_config.opacity_from >= 1.0 {
      return;
    }

    // Stationary styles keep the surrogate at target position; slide styles
    // offset the start one full window dimension off-screen.
    let start_rect = if is_stationary {
      target_rect.clone()
    } else {
      Self::compute_transition_start_rect(&target_rect, &anim_config.style)
    };

    let mut anim = WindowAnimationState::new_movement(
      start_rect.clone(),
      target_rect.clone(),
      anim_config.duration_ms,
      anim_config.easing.clone(),
    );

    // Zoom open does NOT auto-fade — the surrogate is fully opaque so the
    // small thumbnail is immediately visible as it grows. Fade-in while zooming
    // makes the initial frames invisible (opacity=0 + tiny size = nothing to
    // see), which is why it felt unsmooth. Users can still set opacity_from
    // explicitly to combine fade with zoom.
    let effective_opacity_from = anim_config.opacity_from;

    if effective_opacity_from < 1.0 {
      let effect_frac = effect_opacity as f32 / 255.0;
      let start_frac = effective_opacity_from.clamp(0.0, 1.0) * effect_frac;
      anim.start_opacity = Some(OpacityValue(start_frac));
      anim.target_opacity = Some(OpacityValue(effect_frac));
    }

    // Cloak zoom windows immediately so the real window never appears at full
    // size before the surrogate takes over. Non-zoom styles are cloaked later
    // in the Frozen branch of platform_sync (on the first frame).
    if is_zoom {
      let _ = native_window.set_cloaked(true);
    }

    match ResizeSession::begin(
      native_window.hwnd(),
      &start_rect,
      &target_rect,
      None,
      effect_opacity,
      false,
    ) {
      Ok(mut session) => {
        session.zoom = is_zoom;
        let initial_opacity_u8 = (effective_opacity_from.clamp(0.0, 1.0)
          * effect_opacity as f32)
          .round() as u8;
        if effective_opacity_from < 1.0 {
          session.update(&start_rect, initial_opacity_u8);
        }
        // For zoom: the drive loop handles the first frame. update_zoom_fade
        // is NOT called here so the surrogate stays hidden until the first
        // animation tick sets the correct progress.
        self.animations.insert(window_id, anim);
        self.resize_sessions.insert(window_id, session);
        if !is_stationary {
          self.slide_in_monitor_rects.insert(window_id, monitor_rect);
        }
      }
      Err(err) => {
        // Undo early cloak so the window doesn't disappear permanently.
        if is_zoom {
          let _ = native_window.set_cloaked(false);
        }
        tracing::warn!(
          "Failed to begin open animation for {window_id}: {err}."
        );
      }
    }
  }

  /// Starts a close animation for a window.
  ///
  /// The real window is expected to be already cloaked by the caller. The
  /// surrogate style is determined by `window_close.style`:
  /// - `Fade`/`Zoom`: surrogate stays at `current_rect`, fades/zooms out.
  /// - Slide styles: surrogate slides off the corresponding screen edge while
  ///   fading. The real window is never repositioned during a close animation.
  ///
  /// When the animation completes, `update_internal` sends `WM_CLOSE` and
  /// unmanages the window. No-ops if a close animation is already active.
  #[cfg(target_os = "windows")]
  pub fn start_close_animation(
    &mut self,
    window_id: Uuid,
    current_rect: Rect,
    effect_opacity: u8,
    config: &UserConfig,
    native_window: &NativeWindow,
  ) {
    if self.pending_close_windows.contains_key(&window_id) {
      return;
    }

    let anim_config = &config.value.animations.window_close;
    let is_zoom = anim_config.style == WindowTransitionStyle::Zoom;
    let is_stationary = anim_config.style.is_stationary();

    // For slide-out, the surrogate travels from current_rect to an off-screen
    // target. The real window stays at current_rect throughout.
    let target_rect = if is_stationary {
      current_rect.clone()
    } else {
      Self::compute_transition_start_rect(&current_rect, &anim_config.style)
    };

    let mut anim = WindowAnimationState::new_movement(
      current_rect.clone(),
      target_rect.clone(),
      anim_config.duration_ms,
      anim_config.easing.clone(),
    );

    let effect_frac = effect_opacity as f32 / 255.0;
    let target_frac = anim_config.opacity_to.clamp(0.0, 1.0) * effect_frac;
    anim.start_opacity = Some(OpacityValue(effect_frac));
    anim.target_opacity = Some(OpacityValue(target_frac));

    match ResizeSession::begin(
      native_window.hwnd(),
      &current_rect,
      &target_rect,
      None,
      effect_opacity,
      false,
    ) {
      Ok(mut session) => {
        // Show the surrogate immediately — the real window is already cloaked.
        session.show();
        session.zoom = is_zoom;
        self.animations.insert(window_id, anim);
        self.resize_sessions.insert(window_id, session);
        self.pending_close_windows
          .insert(window_id, native_window.hwnd().0);
      }
      Err(err) => {
        tracing::warn!(
          "Failed to begin close animation for {window_id}: {err}."
        );
      }
    }
  }

  /// Computes the off-screen rect for a slide open/close transition.
  ///
  /// For open (`start_open_animation`): returns the start rect positioned
  /// off-screen, one full window dimension outside the target edge. The
  /// surrogate slides from this rect to `base`.
  ///
  /// For close (`start_close_animation`): returns the off-screen target rect
  /// so the surrogate slides from `base` (the window's current position) to
  /// off-screen.
  ///
  /// `SlideRight` → exits/enters from the right edge;
  /// `SlideLeft` → left edge; `SlideTop` → top; `SlideBottom` → bottom.
  #[cfg(target_os = "windows")]
  fn compute_transition_start_rect(
    base: &Rect,
    style: &WindowTransitionStyle,
  ) -> Rect {
    let w = base.width();
    let h = base.height();

    let (x, y) = match style {
      WindowTransitionStyle::SlideRight => (base.x() + w, base.y()),
      WindowTransitionStyle::SlideLeft => (base.x() - w, base.y()),
      WindowTransitionStyle::SlideTop => (base.x(), base.y() - h),
      WindowTransitionStyle::SlideBottom => (base.x(), base.y() + h),
      // Stationary styles never call this function.
      _ => (base.x(), base.y()),
    };

    Rect::from_xy(x, y, w, h)
  }

  /// Hides the workspace-switch surrogate thumbnail for a single window in
  /// `pending_ws_cleanup`.
  ///
  /// Called immediately after `set_cloaked(false)` for each incoming window
  /// so the surrogate thumbnail disappears at the same DWM composition event
  /// as the window uncloak, eliminating the double-blend frame that would
  /// occur if thumbnail hide were deferred until after all windows are
  /// processed.
  #[cfg(target_os = "windows")]
  pub fn hide_pending_ws_cleanup_surrogate(&mut self, window_id: Uuid) {
    let Some(ref mut ws) = self.pending_ws_cleanup else {
      return;
    };
    if let Some(entry) = ws.windows.get_mut(&window_id) {
      if let Some(ref mut s) = entry.surrogate {
        s.hide_thumbnail();
      }
    }
  }

  /// Applies the configured effect opacity to all outgoing workspace-switch
  /// surrogates.
  ///
  /// Called after the outgoing real windows have been cloaked so the
  /// thumbnail opacity transitions from the fully-opaque `show_initial` state
  /// to the configured effect opacity without causing a double-blend frame.
  #[cfg(target_os = "windows")]
  pub fn apply_outgoing_surrogate_opacities(&mut self) {
    let Some(ref mut ws) = self.workspace_switch else {
      return;
    };
    for entry in ws.windows.values_mut() {
      if !entry.is_incoming {
        if let Some(ref mut s) = entry.surrogate {
          s.apply_effect_opacity();
        }
      }
    }
  }

}

