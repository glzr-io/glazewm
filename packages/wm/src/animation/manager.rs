use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
  time::{Duration, Instant},
};

/// Fraction of the monitor's vblank period to lead the animation clock by.
///
/// After `IDXGIOutput::WaitForVBlank` returns, the surrogate update written
/// this tick is not composited by DWM until the *next* vblank — up to one
/// full frame period later. Advancing the animation clock by a fraction of
/// that period makes the computed position align with where it will be when
/// DWM actually presents it, rather than systematically lagging behind.
///
/// Expressed as a fraction of the per-monitor frame period read live from
/// the installed `DxgiVsyncWaiter`, so the compensation scales correctly
/// across monitors with different refresh rates (e.g. 60 Hz + 175 Hz).
/// `0.5` leads by half a frame: a balance between under-compensating (motion
/// trails the cursor) and over-compensating (motion arrives early). Tune
/// here if the slide feels laggy (increase toward `1.0`) or rushed (decrease
/// toward `0.0`).
#[cfg(target_os = "windows")]
const VSYNC_LEAD_FRACTION: f32 = 0.5;

/// Pipeline latency estimate from vsync wake to DWM composition pickup.
///
/// The animation timer thread records the `Instant` at which
/// `IDXGIOutput::WaitForVBlank` returns. By the time `update_internal` runs
/// and calls `DwmUpdateThumbnailProperties`, roughly this many microseconds
/// have elapsed (Tokio scheduling + compute). Using vsync_time +
/// `VSYNC_PIPELINE_OFFSET_US` as "now" shifts the computed position forward
/// to where it will be when DWM actually composites, eliminating the
/// systematic one-pipeline-delay lag on high-Hz monitors. Used by the
/// iris-wipe driver, which has no per-switch vblank period to lead by.
#[cfg(target_os = "windows")]
const VSYNC_PIPELINE_OFFSET_US: u64 = 1_500;

/// Maximum residual slide distance, in pixels, at which a workspace-switch
/// completes early.
///
/// Decelerating easing curves spend a large fraction of their wall-clock
/// duration covering the final sliver of distance, which looks "stuck" at
/// the destination. Completing early avoids that crawl, but the surrogate is
/// then snapped the remaining distance to its target in a single frame before
/// the real windows uncloak. Gating that completion on a fixed *pixel*
/// distance (rather than a fixed progress fraction) keeps the snap below one
/// pixel regardless of slide distance or duration, so it is imperceptible —
/// a fixed 1% fraction would snap ~34px across a 3440px monitor.
#[cfg(target_os = "windows")]
const WS_COMPLETE_THRESHOLD_PX: f32 = 1.5;

/// Grace period held at the start of a window-open animation before the
/// surrogate begins revealing the window.
///
/// A freshly created window has often not painted its first frame when the
/// open animation starts, so its DWM thumbnail is momentarily blank/black —
/// producing a black box that slides in and "pops" to real content at the
/// end. Holding the animation at progress `0.0` for this period (the
/// surrogate stays off-screen for slide/zoom and fully transparent for fade)
/// gives the app time to paint, so the slide reveals real content from the
/// first visible frame. Implemented via `WindowAnimationState::start_delay`,
/// which is measured from the first rendered frame, so the app gets this long
/// *after* the first animation tick to paint. Roughly two frames at 60 Hz —
/// long enough to cover the typical first-paint latency without a perceptible
/// delay in the window appearing.
#[cfg(target_os = "windows")]
const OPEN_PAINT_GRACE: Duration = Duration::from_millis(30);

/// Duration of the surrogate fade-out at animation completion.
///
/// After the real window is uncloaked beneath the (pixel-aligned, live)
/// surrogate, the surrogate's opacity ramps to zero over this period so
/// shadow, border, and late-repaint differences blend in rather than
/// switching in a single composition.
#[cfg(target_os = "windows")]
const SESSION_FADE_OUT: Duration = Duration::from_millis(100);

/// Maximum mid-animation handoff lead in milliseconds.
///
/// The actual lead is `duration_ms * 0.35`, clamped to
/// `[50, HANDOFF_LEAD_MAX_MS]`. Scaling by duration keeps the handoff near
/// the end of the visual travel regardless of easing speed; the cap ensures
/// apps always get ≥50 ms to repaint at the new size before uncloaking.
#[cfg(target_os = "windows")]
const HANDOFF_LEAD_MAX_MS: u64 = 100;

/// How long a sampled surrogate backdrop color stays valid per window.
///
/// Sampling costs two GPU→CPU `BitBlt` readbacks on the WM thread; caching
/// removes them from the keypress path during bursts of relayouts. App
/// background colors change rarely (theme switches), so a stale hit is a
/// cosmetic-only risk bounded by this TTL.
#[cfg(target_os = "windows")]
const EDGE_COLOR_CACHE_TTL: Duration = Duration::from_secs(30);

/// Cache-size threshold above which stale edge-color entries are pruned.
#[cfg(target_os = "windows")]
const EDGE_COLOR_CACHE_PRUNE_LEN: usize = 128;

use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::{
  EasingFunction, WindowTransitionStyle,
  WorkspaceSwitchDirection, WorkspaceSwitchStyle,
};
use wm_platform::{NativeWindow, OpacityValue, Rect};
#[cfg(target_os = "windows")]
use wm_platform::{
  Color, CornerStyle, DxgiVsyncWaiter, NativeIrisOverlay,
  NativeWindowWindowsExt, ResizeSession, SessionOptions, SurrogateBatch,
  WorkspaceSurrogate,
};

use crate::{
  animation::state::WindowAnimationState,
  commands::general::platform_sync,
  traits::CommonGetters,
  user_config::UserConfig,
  wm_state::WmState,
};

/// A single entry in the surrogate update queue built each redraw pass.
#[cfg(target_os = "windows")]
struct PendingSurrogateUpdate {
  window_id: Uuid,
  rect: Rect,
  opacity: u8,
  /// `true` when `remaining_at` ≤ the proportional handoff lead.
  handoff: bool,
}

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
  /// Motion style (slide, fade, or zoom).
  style: WorkspaceSwitchStyle,
  /// Slide axis (horizontal or vertical). Only used when `style` is `Slide`.
  slide_direction: WorkspaceSwitchDirection,
  /// Workspace ordering direction: `+1` = target workspace is higher-index
  /// (incoming from the far edge, outgoing to the near edge). `-1` = opposite.
  order_direction: i32,
  /// Left x-coordinate of the animation monitor in screen pixels.
  monitor_x: i32,
  /// Width of the animation monitor in screen pixels.
  monitor_width: i32,
  /// Top y-coordinate of the animation monitor in screen pixels.
  monitor_y: i32,
  /// Height of the animation monitor in screen pixels.
  monitor_height: i32,
  /// Effective horizontal slide travel distance in screen pixels.
  ///
  /// Less than `monitor_width` by the sum of the outgoing workspace's
  /// trailing gap and the incoming workspace's leading gap (both equal to
  /// `outer_gap` in a standard config). This makes the two workspace panels
  /// start adjacent with no visible seam between them.
  slide_distance_h: i32,
  /// Effective vertical slide travel distance in screen pixels.
  ///
  /// Mirrors `slide_distance_h` on the y-axis.
  slide_distance_v: i32,
  /// Whether `start_time` has been re-anchored to a vsync timestamp.
  ///
  /// The clock is provisionally anchored on the wall-clock cold-start tick,
  /// then re-anchored once to the first real vblank so all vsync-driven
  /// frames share a single origin. Stays `false` (wall-clock only) if no
  /// vblank signal ever arrives.
  vsync_anchored: bool,
  /// Scale applied to the whole workspace during slide transitions.
  ///
  /// Derived from `WorkspaceSwitchAnimationConfig::zoom_factor`. `0.0` means
  /// no zoom (plain slide). The outgoing workspace scales from `1.0` to
  /// `1.0 - zoom_factor`; the incoming from `1.0 - zoom_factor` to `1.0`.
  zoom_factor: f32,
}

/// State for an active iris-wipe workspace transition.
///
/// Unlike the per-window slide, the iris wipe uses a single frozen snapshot
/// overlay: the incoming workspace is switched in normally (instantly)
/// underneath, and a growing circular hole in the overlay reveals it. No
/// per-window surrogates are involved.
#[cfg(target_os = "windows")]
struct IrisSwitchState {
  /// Snapshot overlay shown on top of the (already switched) real windows.
  overlay: NativeIrisOverlay,
  /// Circle origin (screen pixels) from which the hole grows.
  origin_x: i32,
  origin_y: i32,
  /// Radius (px) at which the hole fully covers the monitor.
  max_radius: i32,
  /// Time of the first rendered frame, lazily set on the first tick (mirrors
  /// `WorkspaceSwitchState::start_time`).
  start_time: Option<Instant>,
  /// Total animation duration.
  duration: Duration,
  /// Easing applied to raw elapsed-time progress.
  easing: EasingFunction,
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
  /// Whether the animation timer thread is currently ticking (vs. parked).
  ///
  /// Acts as the gate for the persistent timer thread: set `true` to start a
  /// ticking phase, `false` to send it back to parking.
  animation_timer_running: Arc<AtomicBool>,
  /// Handle to the persistent animation timer thread.
  ///
  /// Spawned lazily on the first animation and kept for the process lifetime,
  /// parking between animations rather than being re-spawned each time. This
  /// removes the per-animation cost of thread creation + priority/MMCSS setup
  /// from the input-to-first-frame latency path.
  timer_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
  /// Signals the persistent timer thread to exit. Set on drop.
  timer_shutdown: Arc<AtomicBool>,
  /// DXGI vsync waiter for the animation monitor.
  ///
  /// When `Some`, the timer thread calls `WaitForVBlank` on this output
  /// instead of `DwmFlush`. This gives a full frame period after each vsync
  /// to update surrogates, regardless of which monitor is the Windows primary.
  /// Cleared on workspace-switch completion.
  #[cfg(target_os = "windows")]
  animation_timer_vsync: Arc<Mutex<Option<DxgiVsyncWaiter>>>,
  /// Timestamp of the most recent `IDXGIOutput::WaitForVBlank` wake-up.
  ///
  /// Written by the timer thread immediately after vsync fires. Read by
  /// `update_internal` to compute animation progress at a predictive
  /// timestamp (vsync time + a fraction of the vblank period; see
  /// [`VSYNC_LEAD_FRACTION`]) rather than `Instant::now()`, compensating for
  /// the pipeline delay between vsync wake and DWM composition.
  #[cfg(target_os = "windows")]
  animation_vsync_time: Arc<Mutex<Option<Instant>>>,
  /// Active resize sessions keyed by window ID.
  ///
  /// A session is created when a movement/resize animation starts with
  /// `use_surrogate = true` and is destroyed once the animation completes
  /// and the real window has been moved to its final position.
  #[cfg(target_os = "windows")]
  pub(crate) resize_sessions: HashMap<Uuid, ResizeSession>,
  /// Recently sampled surrogate backdrop colors keyed by window handle.
  ///
  /// Lets `ResizeSession::begin` skip its two-`BitBlt` screen sample for
  /// windows resized again within [`EDGE_COLOR_CACHE_TTL`], removing the
  /// GPU→CPU readback burst when a relayout begins many sessions in the
  /// same keypress.
  #[cfg(target_os = "windows")]
  edge_color_cache: HashMap<isize, (Color, Instant)>,
  /// Surrogate updates queued during this redraw pass; committed atomically by
  /// [`flush_surrogate_updates`] so adjacent surrogates land in the same DWM
  /// composition frame.
  ///
  /// [`flush_surrogate_updates`]: AnimationManager::flush_surrogate_updates
  #[cfg(target_os = "windows")]
  pending_surrogate_updates: Vec<PendingSurrogateUpdate>,
  /// Sessions that have been removed from `resize_sessions` after their
  /// animation completed but must outlive the final `platform_sync` call
  /// that repositions the real window. Keyed by window ID so the final
  /// redraw can detect that `pre_commit` already positioned the window.
  ///
  /// The `Option<Instant>` is the fade-out start time: `None` until the
  /// real window has been uncloaked beneath the surrogate, then set on the
  /// first cleanup tick. Entries are dropped once the fade completes.
  #[cfg(target_os = "windows")]
  pub(crate) pending_session_cleanup:
    Vec<(Uuid, Option<Instant>, ResizeSession)>,
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
  /// Active iris-wipe workspace transition, or `None` when idle.
  #[cfg(target_os = "windows")]
  iris_switch: Option<IrisSwitchState>,
}

impl Drop for AnimationManager {
  /// Signals the persistent timer thread to exit.
  ///
  /// Sets the shutdown flag, clears the ticking gate, and unparks the thread
  /// so it observes the shutdown and returns. The handle is dropped without
  /// joining so shutdown never blocks — a thread mid-`WaitForVBlank` on a
  /// sleeping monitor could otherwise stall the join indefinitely. The thread
  /// exits on its own and is reaped by the OS at process exit.
  fn drop(&mut self) {
    self.timer_shutdown.store(true, Ordering::Relaxed);
    self.animation_timer_running.store(false, Ordering::Relaxed);
    if let Some(handle) =
      self.timer_thread.lock().expect("animation mutex poisoned").take()
    {
      handle.thread().unpark();
    }
  }
}

impl AnimationManager {
  /// Creates a new `AnimationManager`.
  pub fn new(animation_tick_tx: mpsc::UnboundedSender<()>) -> Self {
    Self {
      animations: HashMap::new(),
      animation_tick_tx,
      animation_timer_running: Arc::new(AtomicBool::new(false)),
      timer_thread: Mutex::new(None),
      timer_shutdown: Arc::new(AtomicBool::new(false)),
      #[cfg(target_os = "windows")]
      animation_timer_vsync: Arc::new(Mutex::new(None)),
      #[cfg(target_os = "windows")]
      animation_vsync_time: Arc::new(Mutex::new(None)),
      #[cfg(target_os = "windows")]
      resize_sessions: HashMap::new(),
      #[cfg(target_os = "windows")]
      edge_color_cache: HashMap::new(),
      #[cfg(target_os = "windows")]
      pending_surrogate_updates: Vec::new(),
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
      #[cfg(target_os = "windows")]
      iris_switch: None,
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

  /// Returns `true` if a close animation is in flight for the given window.
  #[cfg(target_os = "windows")]
  pub fn has_close_animation(&self, window_id: &Uuid) -> bool {
    self.pending_close_windows.contains_key(window_id)
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
        self.pending_session_cleanup.push((*id, None, session));
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
    #[cfg(target_os = "windows")]
    if self.iris_switch.is_some() {
      return true;
    }
    // Completed sessions fading out still need ticks to advance the fade.
    #[cfg(target_os = "windows")]
    if !self.pending_session_cleanup.is_empty() {
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
    sessions
      .extend(self.pending_session_cleanup.drain(..).map(|(_, _, s)| s));
    self.pending_surrogate_updates.clear();
    self.workspace_switch = None;
    self.pending_ws_cleanup = None;
    *self.animation_timer_vsync.lock().expect("animation mutex poisoned") = None;
    *self.animation_vsync_time.lock().expect("animation mutex poisoned") = None;
    // On WM shutdown close-animation windows are left open — only clear
    // the tracking state without sending WM_CLOSE.
    self.pending_close_windows.clear();
    // Drop the iris overlay (if any); the real windows are already at their
    // final positions, so tearing it down simply reveals them.
    self.iris_switch = None;
    sessions
  }

  /// Starts a ticking phase of the persistent animation timer thread.
  ///
  /// The timer thread is spawned once (on the first animation) and parked
  /// between animations rather than re-created each time, so the cost of
  /// thread creation and priority/MMCSS setup never lands on the
  /// input-to-first-frame latency path. This call wakes the parked thread (or
  /// spawns it the first time) when there are active animations and the
  /// thread is not already ticking.
  ///
  /// The thread uses a two-tier vsync strategy on Windows:
  ///
  /// 1. **`IDXGIOutput::WaitForVBlank`** — when a `DxgiVsyncWaiter` is
  ///    installed, waits for the animation monitor's specific vblank signal.
  ///    Per-monitor and reliable at any Hz.
  /// 2. **`DwmFlush`** — fallback when no waiter is installed, aligning to
  ///    the primary monitor's composition cycle.
  ///
  /// On non-Windows, `DwmFlush` is a no-op so a fixed 60 fps sleep is used.
  pub fn ensure_timer_running(&self) {
    if !self.has_active_animations() {
      return;
    }

    // Idempotent: if the thread is already ticking, there is nothing to do.
    // The swap also claims the idle -> ticking transition so only one caller
    // wakes the thread.
    if self.animation_timer_running.swap(true, Ordering::Relaxed) {
      return;
    }

    let mut guard =
      self.timer_thread.lock().expect("animation mutex poisoned");
    match guard.as_ref() {
      // Wake the parked thread to begin a new ticking phase.
      Some(handle) => handle.thread().unpark(),
      // First animation: spawn the persistent thread.
      None => {
        if let Some(handle) = self.spawn_timer_thread() {
          *guard = Some(handle);
        }
      }
    }
  }

  /// Spawns the persistent animation timer thread.
  ///
  /// Returns the join handle, or `None` if the OS thread could not be
  /// created. On failure the ticking gate is reset so a later
  /// [`ensure_timer_running`] call retries the spawn.
  ///
  /// [`ensure_timer_running`]: AnimationManager::ensure_timer_running
  fn spawn_timer_thread(&self) -> Option<std::thread::JoinHandle<()>> {
    let tx = self.animation_tick_tx.clone();
    let running = self.animation_timer_running.clone();
    let shutdown = self.timer_shutdown.clone();
    #[cfg(target_os = "windows")]
    let vsync_waiter = self.animation_timer_vsync.clone();
    #[cfg(target_os = "windows")]
    let vsync_time = self.animation_vsync_time.clone();

    // Spawn a real OS thread (not a Tokio task) so blocking vsync waits do
    // not stall the async runtime.
    let result = std::thread::Builder::new()
      .name("glazewm-anim-tick".into())
      .spawn(move || {
        // Elevate scheduling priority once for the thread's whole lifetime.
        // MMCSS "DisplayPostProcessing" gives near-real-time guarantees beyond
        // THREAD_PRIORITY_HIGHEST, matching the scheduling class used by DWM
        // and video renderers. Falls back gracefully to
        // THREAD_PRIORITY_HIGHEST if avrt.dll is unavailable. The persistent
        // thread keeps the registration across animations rather than
        // re-acquiring it per switch; while parked it consumes no CPU.
        wm_platform::set_thread_priority_highest();
        #[cfg(target_os = "windows")]
        let _mmcss = wm_platform::try_set_thread_mmcss();

        loop {
          // Idle: park until a ticking phase begins (or shutdown). A `while`
          // loop re-checks the gate to absorb spurious wake-ups and any
          // buffered unpark token from a just-ended phase.
          while !running.load(Ordering::Relaxed) {
            if shutdown.load(Ordering::Relaxed) {
              return;
            }
            std::thread::park();
          }
          if shutdown.load(Ordering::Relaxed) {
            return;
          }

          // Send an immediate tick so the first animation frame begins
          // without waiting for the next vblank. Without this the surrogate
          // is frozen at its start position for up to one full frame period
          // (~16 ms at 60 Hz, ~5.7 ms at 175 Hz) before any movement begins.
          if tx.send(()).is_err() {
            return;
          }

          // Ticking phase: drive frames until the gate clears.
          while running.load(Ordering::Relaxed) {
            // Per-monitor IDXGIOutput::WaitForVBlank during workspace-switch.
            // Clone under the lock so the wait runs without holding it —
            // cleanup can clear the Arc without blocking an in-progress wait.
            // Record the wake-up time immediately after vsync fires so
            // `update_internal` can compute phase-accurate animation progress.
            #[cfg(target_os = "windows")]
            let dxgi_waited = {
              let waiter = vsync_waiter
                .lock()
                .expect("animation mutex poisoned")
                .clone();
              let waited = waiter.map(|w| w.wait()).unwrap_or(false);
              if waited {
                *vsync_time.lock().expect("animation mutex poisoned") =
                  Some(Instant::now());
              }
              waited
            };
            #[cfg(not(target_os = "windows"))]
            let dxgi_waited = false;

            if !dxgi_waited {
              // DwmFlush for window move/resize animations (no
              // workspace-switch active). On non-Windows this is a no-op, so
              // a fixed 60 fps sleep paces the loop.
              wm_platform::dwm_flush();
              #[cfg(not(target_os = "windows"))]
              std::thread::sleep(std::time::Duration::from_micros(16_667));
            }

            if tx.send(()).is_err() {
              return;
            }
          }
          // Gate cleared: loop back to park until the next animation.
        }
        // `_mmcss` is dropped on return, reverting the MMCSS registration.
      });

    match result {
      Ok(handle) => Some(handle),
      Err(err) => {
        tracing::warn!("Failed to spawn animation tick thread: {err}.");
        // Reset the gate so a later call retries the spawn.
        self.animation_timer_running.store(false, Ordering::Relaxed);
        None
      }
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

    // Drive close surrogates directly. These windows have been detached from
    // the layout tree when the close animation started, so they are not
    // queued for redraw by the loop above and cannot be driven through
    // `platform_sync`. We replicate the same per-frame update logic used
    // inside `start_animation_if_needed` for surrogate sessions.
    #[cfg(target_os = "windows")]
    {
      let close_in_progress: Vec<Uuid> = state
        .animation_manager
        .pending_close_windows
        .keys()
        .filter(|id| {
          state
            .animation_manager
            .animations
            .get(id)
            .map_or(false, |a| !a.is_complete())
        })
        .copied()
        .collect();

      for id in &close_in_progress {
        let is_zoom = state
          .animation_manager
          .resize_sessions
          .get(id)
          .map(|s| s.zoom)
          .unwrap_or(false);

        // Extract values before taking a mutable borrow on resize_sessions.
        let anim_data =
          state.animation_manager.animations.get(id).map(|a| {
            let (rect, opacity) = a.current_state();
            let progress = a.eased_progress();
            (rect, opacity, progress)
          });

        let Some((current_rect, opacity, progress)) = anim_data else {
          continue;
        };
        let opacity_u8 =
          opacity.map(|o| o.to_alpha()).unwrap_or(u8::MAX);

        if is_zoom {
          if let Some(session) =
            state.animation_manager.resize_sessions.get_mut(id)
          {
            session.update_zoom_fade(1.0 - progress, opacity_u8);
          }
        } else if let Some(session) =
          state.animation_manager.resize_sessions.get_mut(id)
        {
          session.update(&current_rect, opacity_u8);
        }
      }
    }

    // Finalize completed close animations before `remove_completed_animations`
    // so that their sessions are dropped directly (not moved to
    // `pending_session_cleanup`) and `platform_sync` never attempts to
    // reposition or uncloak these windows.
    //
    // The window was already detached from the layout tree when the close
    // animation started, so `WM_CLOSE` is sent via the stored HWND rather
    // than through the container tree.
    #[cfg(target_os = "windows")]
    {
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
        let hwnd = state
          .animation_manager
          .pending_close_windows
          .get(&id)
          .copied();

        // Drop surrogate and animation directly — bypasses
        // `pending_session_cleanup` so `platform_sync` does not attempt to
        // reposition or uncloak the window.
        state.animation_manager.animations.remove(&id);
        state.animation_manager.resize_sessions.remove(&id);
        state.animation_manager.pending_close_windows.remove(&id);

        // Reconstruct a `NativeWindow` from the stored HWND and send
        // `WM_CLOSE` to destroy the OS window. The layout was already
        // updated when the animation started, so no `unmanage_window` call
        // is needed here.
        if let Some(handle) = hwnd {
          let native = NativeWindow::from_handle(handle);
          if let Err(err) = native.close() {
            tracing::warn!(
              "Failed to send WM_CLOSE for window {id}: {err}."
            );
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
    // the real window to its target position and uncloak it.
    for window_id in &completed_ids {
      if let Some(container) = state.container_by_id(*window_id) {
        if let Ok(window) = container.as_window_container() {
          state.pending_sync.queue_container_to_redraw(window);
        }
      }
    }

    // Re-apply any focus change that was deferred while the now-completed
    // resize surrogates were active. `sync_focus` skips `SetForegroundWindow`
    // while a resize session is live to prevent the OS from asynchronously
    // removing the DWM cloak and triggering a costly re-cloak on the next tick.
    // Queuing here ensures the focus transfer happens in the same `platform_sync`
    // that uncloak the windows.
    #[cfg(target_os = "windows")]
    if !completed_ids.is_empty() {
      state.pending_sync.queue_focus_change();
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
      use crate::animation::engine::{
        animation_progress_at, apply_easing,
      };

      // Compute the predictive vsync timestamp before taking a mutable borrow
      // on `workspace_switch`. `predictive_vsync_now` reads from the installed
      // waiter, so the period is always current — no stale cached field.
      let ws_vsync_now =
        state.animation_manager.predictive_vsync_now();

      if let Some(ws) = &mut state.animation_manager.workspace_switch {
        // Anchor the animation clock to the first vsync-aligned tick so every
        // inter-frame step is measured on the same clock. Falls back to the
        // wall clock so progress always advances if no vblank signal arrives.
        let now = match ws_vsync_now {
          Some(vsync) => {
            if !ws.vsync_anchored {
              ws.start_time = Some(vsync);
              ws.vsync_anchored = true;
            }
            vsync
          }
          None => Instant::now(),
        };
        let start = *ws.start_time.get_or_insert(now);
        let raw_progress = animation_progress_at(start, ws.duration, now);
        let eased = apply_easing(raw_progress, &ws.easing);

        // Complete early once the surrogate is within
        // `WS_COMPLETE_THRESHOLD_PX` of its target for non-overshooting
        // curves — decelerating easing spends a large fraction of wall time
        // covering the final sliver of distance, which looks "stuck" at the
        // destination. Gating on residual *pixels* rather than a fixed
        // progress fraction keeps the completion-frame snap sub-pixel for any
        // slide distance or duration. Slide styles use their axis travel
        // distance; fade/zoom have no positional travel, so fall back to a
        // 99% fraction (a 1% opacity/scale snap is invisible). Overshooting
        // curves always run to full wall-clock duration to preserve bounce.
        let ws_done = if ws.easing.can_overshoot() {
          raw_progress >= 1.0
        } else if raw_progress >= 1.0 {
          true
        } else {
          let slide_px = match ws.style {
            WorkspaceSwitchStyle::Slide => match ws.slide_direction {
              WorkspaceSwitchDirection::Horizontal => ws.slide_distance_h,
              WorkspaceSwitchDirection::Vertical => ws.slide_distance_v,
            },
            // Iris never runs through the per-window slide path (it is driven
            // separately via `iris_switch`), but the match must stay exhaustive.
            WorkspaceSwitchStyle::Fade
            | WorkspaceSwitchStyle::Zoom
            | WorkspaceSwitchStyle::Iris => 0,
          };
          if slide_px > 0 {
            #[allow(clippy::cast_precision_loss)]
            let remaining_px = (1.0 - eased) * slide_px as f32;
            remaining_px <= WS_COMPLETE_THRESHOLD_PX
          } else {
            eased >= 0.99
          }
        };

        // When completing early (eased < 1.0), snap surrogates to 1.0 so
        // they sit exactly at the final window position. Without this, a
        // ~1% gap between surrogate and the just-uncloaked real window
        // exposes the desktop for one frame.
        let eased_final = if ws_done { 1.0 } else { eased };

        for entry in ws.windows.values_mut() {
          if let Some(ref mut s) = entry.surrogate {
            // At completion, hide outgoing surrogates immediately. They have
            // already slid fully off-screen, but hiding the thumbnail outright
            // guarantees nothing lingers for the final composition frame
            // before the real windows are uncloaked.
            if ws_done && !entry.is_incoming {
              s.hide_thumbnail();
              continue;
            }
            match ws.style {
              WorkspaceSwitchStyle::Slide => {
                match ws.slide_direction {
                  WorkspaceSwitchDirection::Horizontal => {
                    if ws.zoom_factor > 0.0 {
                      s.update_slide_zoom_horizontal(
                        eased_final,
                        entry.is_incoming,
                        ws.order_direction,
                        ws.monitor_x,
                        ws.monitor_width,
                        ws.monitor_y,
                        ws.monitor_height,
                        ws.slide_distance_h,
                        ws.zoom_factor,
                      );
                    } else {
                      s.update_slide_horizontal(
                        eased_final,
                        entry.is_incoming,
                        ws.order_direction,
                        ws.monitor_x,
                        ws.monitor_width,
                        ws.slide_distance_h,
                      );
                    }
                  }
                  WorkspaceSwitchDirection::Vertical => {
                    if ws.zoom_factor > 0.0 {
                      s.update_slide_zoom_vertical(
                        eased_final,
                        entry.is_incoming,
                        ws.order_direction,
                        ws.monitor_x,
                        ws.monitor_width,
                        ws.monitor_y,
                        ws.monitor_height,
                        ws.slide_distance_v,
                        ws.zoom_factor,
                      );
                    } else {
                      s.update_slide_vertical(
                        eased_final,
                        entry.is_incoming,
                        ws.order_direction,
                        ws.monitor_y,
                        ws.monitor_height,
                        ws.slide_distance_v,
                      );
                    }
                  }
                }
              }
              WorkspaceSwitchStyle::Fade => {
                s.update_fade(eased_final, entry.is_incoming);
              }
              WorkspaceSwitchStyle::Zoom => {
                s.update_zoom(eased_final, entry.is_incoming);
              }
              // Iris is driven by a separate snapshot overlay (see
              // `iris_switch`), never by per-window surrogates, so it never
              // reaches this driver.
              WorkspaceSwitchStyle::Iris => {}
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

    // Drive the iris-wipe overlay. The incoming workspace was already switched
    // in normally underneath the overlay; here a growing circular hole reveals
    // it. Uses the same vsync-aligned predictive timestamp as the slide driver.
    #[cfg(target_os = "windows")]
    {
      use crate::animation::engine::{animation_progress_at, apply_easing};

      let iris_done =
        if let Some(iris) = &mut state.animation_manager.iris_switch {
          let start = *iris.start_time.get_or_insert_with(Instant::now);
          let raw_progress = {
            let now = state
              .animation_manager
              .animation_vsync_time
              .lock()
              .expect("animation mutex poisoned")
              .map(|t| {
                t + std::time::Duration::from_micros(VSYNC_PIPELINE_OFFSET_US)
              })
              .unwrap_or_else(Instant::now);
            animation_progress_at(start, iris.duration, now)
          };
          let eased = apply_easing(raw_progress, &iris.easing);
          // Grow the hole from 0 to `max_radius` (which reaches the farthest
          // corner at `eased == 1.0`); the overlay is dropped on the same final
          // frame, so the corners never linger.
          let radius = (eased * iris.max_radius as f32).round() as i32;
          iris.overlay.set_hole(iris.origin_x, iris.origin_y, radius);
          raw_progress >= 1.0
        } else {
          false
        };

      if iris_done {
        // Dropping the overlay destroys the snapshot window, revealing the
        // fully switched incoming workspace beneath.
        state.animation_manager.iris_switch = None;
      }
    }

    if state.pending_sync.has_changes() {
      platform_sync(state, config)?;
    }

    // Fade out pending sessions now that `platform_sync` has moved the real
    // windows to their final positions, then drop them. Dropping a session
    // destroys its surrogate overlay.
    #[cfg(target_os = "windows")]
    {
      // Flush before fading new surrogates or dropping workspace-switch
      // cleanup. The uncloak issued inside `platform_sync` is a DWM
      // attribute change that only takes effect at the next composition —
      // fading or destroying the surrogate without waiting for that frame
      // can produce one composition where the surrogate is dimmed/gone but
      // the real window is still cloaked: a visible blank flash at the end
      // of the animation. The flush guarantees DWM renders one frame with
      // the real window visible (including its compositor shadow, which
      // thumbnails do not capture) while the surrogate still fully covers
      // it.
      let has_new_session_cleanup = state
        .animation_manager
        .pending_session_cleanup
        .iter()
        .any(|(_, fade_start, _)| fade_start.is_none());
      if has_new_session_cleanup
        || state.animation_manager.pending_ws_cleanup.is_some()
      {
        wm_platform::dwm_flush();
      }

      // After `pre_commit` the surrogate is a pixel-aligned live mirror of
      // the uncloaked window beneath it, so ramping its opacity to zero
      // blends shadow/border/late-repaint differences instead of swapping
      // them in a single composition. Entries are dropped once fully faded.
      let fade_now = Instant::now();
      state.animation_manager.pending_session_cleanup.retain_mut(
        |(_, fade_start, session)| {
          let start = *fade_start.get_or_insert(fade_now);
          let progress = fade_now.saturating_duration_since(start).as_secs_f32()
            / SESSION_FADE_OUT.as_secs_f32();
          if progress >= 1.0 {
            return false;
          }
          #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
          let opacity =
            (f32::from(session.effect_opacity) * (1.0 - progress)) as u8;
          session.fade_overlay(opacity);
          true
        },
      );
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

      // All animations are done: clear the move/resize DXGI vsync waiter so a
      // later animation re-selects its own monitor and the timer reverts to
      // DwmFlush when vsync is unavailable. Workspace-switch/iris clear their
      // own waiter above; this also covers move/resize, which has no
      // dedicated completion hook. Safe here because no animation is active.
      #[cfg(target_os = "windows")]
      {
        *state
          .animation_manager
          .animation_timer_vsync
          .lock()
          .expect("animation mutex poisoned") = None;
        *state
          .animation_manager
          .animation_vsync_time
          .lock()
          .expect("animation mutex poisoned") = None;
      }
    }

    Ok(())
  }

  /// Returns the predicted vsync instant: last wake + frame period ×
  /// (1 - `VSYNC_LEAD_FRACTION`).
  ///
  /// Reads the period live from the installed waiter instead of a stale
  /// cached field. Returns `None` when no waiter is installed or no wake has
  /// been recorded yet.
  #[cfg(target_os = "windows")]
  fn predictive_vsync_now(&self) -> Option<Instant> {
    let guard = self.animation_timer_vsync.lock().ok()?;
    let waiter = guard.as_ref()?;
    let period_us = waiter.frame_period_us();
    let last_wake = waiter.last_wake.lock().ok()?.as_ref().copied()?;
    #[allow(
      clippy::cast_precision_loss,
      clippy::cast_possible_truncation,
      clippy::cast_sign_loss
    )]
    let lead = Duration::from_micros(
      (period_us as f64 * f64::from(1.0_f32 - VSYNC_LEAD_FRACTION)) as u64,
    );
    Some(last_wake + lead)
  }

  /// Returns the predictive vsync instant if available, else wall-clock now.
  #[cfg(target_os = "windows")]
  fn predictive_now(&self) -> Instant {
    self.predictive_vsync_now().unwrap_or_else(Instant::now)
  }

  /// Installs or upgrades the vsync waiter to the monitor with handle
  /// `monitor_handle`.
  ///
  /// No-op during workspace or iris switch (they own the waiter). Skips DXGI
  /// enumeration when the window is already on the installed monitor.
  #[cfg(target_os = "windows")]
  fn ensure_waiter_for(&self, monitor_handle: isize) {
    // Don't touch the waiter during switch phases — they manage it.
    if self.workspace_switch.is_some() || self.iris_switch.is_some() {
      return;
    }

    // Fast path: already pacing from this monitor.
    {
      let guard = self
        .animation_timer_vsync
        .lock()
        .unwrap_or_else(|e| e.into_inner());
      if guard
        .as_ref()
        .is_some_and(|w| w.monitor_handle() == monitor_handle)
      {
        return;
      }
    }

    // Upgrade only — never downgrade to a slower refresh rate.
    match DxgiVsyncWaiter::for_monitor(monitor_handle) {
      Ok(new_waiter) => {
        let mut guard = self
          .animation_timer_vsync
          .lock()
          .unwrap_or_else(|e| e.into_inner());
        let should_replace = guard
          .as_ref()
          .map_or(true, |w| new_waiter.frame_period_us() < w.frame_period_us());
        if should_replace {
          tracing::debug!(
            monitor = monitor_handle,
            period_us = new_waiter.frame_period_us(),
            "vsync waiter upgraded"
          );
          *guard = Some(new_waiter);
        }
      }
      Err(err) => {
        tracing::warn!(?err, "failed to create vsync waiter for monitor");
      }
    }
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
  /// `cycle_has_resize` is `true` when any window in the same redraw cycle
  /// changes size. Pure translations then adopt the `window_resize` timing
  /// and easing so adjacent edges stay in lock-step throughout the relayout —
  /// with differing `window_move`/`window_resize` durations, a moved window
  /// would otherwise arrive early and detach from its still-resizing
  /// neighbors.
  ///
  /// Returns [`AnimationPositionResult::Frozen`] while a surrogate overlay
  /// is active so the caller does not reposition the real window on
  /// intermediate frames.
  pub fn start_animation_if_needed(
    &mut self,
    window_id: Uuid,
    is_resize: bool,
    cycle_has_resize: bool,
    target_rect: Rect,
    previous_target: Option<Rect>,
    // Only used on Windows to capture the window for the surrogate.
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    native_window: &NativeWindow,
    // Opacity from window-effects config; used as surrogate opacity when the
    // animation has no per-frame fade component.
    #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
    effect_opacity: u8,
    // Corner style from window-effects config; applied to the surrogate so it
    // matches the real window's rounded corners during the animation.
    #[cfg(target_os = "windows")]
    corner_style: CornerStyle,
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

        // Share the resize timing across the whole cycle when any window in
        // it resizes, keeping all edges in lock-step (see doc comment).
        let use_resize_timing = is_resize
          || (cycle_has_resize
            && config.value.animations.window_resize.enabled);

        let (duration_ms, easing) = if use_resize_timing {
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
          session.update_target(&start_rect, &target_rect);
        } else {
          // Drop any still-fading surrogate from a just-completed animation
          // of this window so two overlays don't stack.
          self
            .pending_session_cleanup
            .retain(|(id, _, _)| id != &window_id);
          let hwnd = native_window.hwnd();
          let cached_edge_color = self.cached_edge_color(hwnd.0);
          let had_cached_color = cached_edge_color.is_some();
          match ResizeSession::begin(
            hwnd,
            &start_rect,
            &target_rect,
            SessionOptions {
              effect_opacity,
              initially_visible: true,
              corner_style,
              place_at_top: true,
              edge_color: cached_edge_color,
            },
          ) {
            Ok(session) => {
              // Only cache on a miss so staleness stays bounded by the TTL
              // rather than sliding forward on every reuse.
              if !had_cached_color {
                if let Some(color) = session.edge_color() {
                  self.remember_edge_color(hwnd.0, color.clone());
                }
              }
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
        // Install or upgrade the vsync waiter to the highest-Hz monitor
        // among active sessions after both the new-session and redirect paths.
        #[cfg(target_os = "windows")]
        self.ensure_waiter_for(DxgiVsyncWaiter::window_monitor(
          native_window.hwnd(),
        ));
      }
    }

    // Evaluate this frame's position at a predictive timestamp so the
    // surrogate aligns with the next DWM composition rather than lagging by
    // one pipeline delay. On non-Windows there is no vsync clock, so this is
    // just `Instant::now()`.
    #[cfg(target_os = "windows")]
    let now = self.predictive_now();
    #[cfg(not(target_os = "windows"))]
    let now = Instant::now();

    // Re-fetch the animation after potentially starting a new one.
    if let Some(animation) = self.get_animation(&window_id) {
      let (current_rect, opacity) = animation.current_state_at(now);

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
          let session = self
            .resize_sessions
            .get_mut(&window_id)
            .expect("resize session must exist after status check");
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
            let session = self
              .resize_sessions
              .get_mut(&window_id)
              .expect("resize session must exist after status check");
            session.update_zoom_fade(forward_progress, opacity_u8);
          } else if let Some(monitor_rect) = monitor_rect {
            session.update_clipped(&current_rect, &monitor_rect, opacity_u8);
          } else {
            // Queue instead of applying immediately: all surrogate
            // repositions in this redraw pass are committed atomically by
            // `flush_surrogate_updates` so adjacent windows' edges land in
            // the same DWM composition frame.
            let handoff =
              self.animations.get(&window_id).map_or(false, |a| {
                // Scale the lead with the animation duration so the handoff
                // stays near the end of the visual travel regardless of
                // easing speed. For expo-out at 150 ms this fires at ~96%
                // visual progress.
                #[allow(
                  clippy::cast_possible_truncation,
                  clippy::cast_sign_loss
                )]
                let lead_ms = (a.duration.as_millis() as f32 * 0.35)
                  .clamp(50.0, HANDOFF_LEAD_MAX_MS as f32)
                  as u64;
                a.remaining_at(now) <= Duration::from_millis(lead_ms)
              });
            self.pending_surrogate_updates.push(PendingSurrogateUpdate {
              window_id,
              rect: current_rect,
              opacity: opacity_u8,
              handoff,
            });
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

  /// Returns `true` when `window_id`'s animation just completed and
  /// `pre_commit` synchronously positioned the real window at `rect`.
  ///
  /// Used by `platform_sync` on the completion redraw to skip the redundant
  /// `SetWindowPos` — its `SWP_FRAMECHANGED` would force a full frame
  /// recalculation and repaint of the window right as it is uncloaked,
  /// producing a visible flash at the end of move/resize animations.
  #[cfg(target_os = "windows")]
  pub fn was_pre_committed_at(
    &self,
    window_id: &Uuid,
    rect: &Rect,
  ) -> bool {
    self
      .pending_session_cleanup
      .iter()
      .any(|(id, _, session)| {
        id == window_id && session.target_rect() == rect
      })
  }

  /// Returns the cached surrogate backdrop color for `hwnd` when still
  /// within [`EDGE_COLOR_CACHE_TTL`].
  #[cfg(target_os = "windows")]
  fn cached_edge_color(&self, hwnd: isize) -> Option<Color> {
    self
      .edge_color_cache
      .get(&hwnd)
      .filter(|(_, sampled_at)| sampled_at.elapsed() < EDGE_COLOR_CACHE_TTL)
      .map(|(color, _)| color.clone())
  }

  /// Caches `color` as `hwnd`'s surrogate backdrop color.
  ///
  /// Prunes stale entries once the cache exceeds
  /// [`EDGE_COLOR_CACHE_PRUNE_LEN`] so closed windows don't accumulate
  /// indefinitely.
  #[cfg(target_os = "windows")]
  fn remember_edge_color(&mut self, hwnd: isize, color: Color) {
    if self.edge_color_cache.len() >= EDGE_COLOR_CACHE_PRUNE_LEN {
      self
        .edge_color_cache
        .retain(|_, (_, sampled_at)| sampled_at.elapsed() < EDGE_COLOR_CACHE_TTL);
    }
    self.edge_color_cache.insert(hwnd, (color, Instant::now()));
  }

  /// Applies all surrogate updates queued during the current redraw pass in
  /// a single `DeferWindowPos` transaction.
  ///
  /// Called at the end of each redraw pass. Committing all repositions
  /// atomically guarantees that adjacent windows' surrogates move in the
  /// same DWM composition frame during multi-window relayouts; sequential
  /// per-surrogate `SetWindowPos` calls can straddle a composition boundary
  /// and let edges visibly desync for a frame.
  #[cfg(target_os = "windows")]
  pub fn flush_surrogate_updates(&mut self) {
    if self.pending_surrogate_updates.is_empty() {
      return;
    }

    let mut batch = SurrogateBatch::new();
    for update in std::mem::take(&mut self.pending_surrogate_updates) {
      if let Some(session) =
        self.resize_sessions.get_mut(&update.window_id)
      {
        if update.handoff {
          session.maybe_handoff();
        }
        session.defer_update(&mut batch, &update.rect, update.opacity);
      }
    }
    batch.commit();
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

  /// Returns `true` when `window_id` is an incoming participant held in
  /// `pending_ws_cleanup` (the one-tick cleanup state after the animation
  /// completes). Used to force synchronous `SetWindowPos` before uncloaking
  /// so the window is already at its target position when revealed.
  #[cfg(target_os = "windows")]
  pub fn is_pending_ws_cleanup_incoming(&self, window_id: &Uuid) -> bool {
    self
      .pending_ws_cleanup
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
  ///
  /// `monitor_handle` is the `HMONITOR` of the animation monitor, used to
  /// look up the `IDXGIOutput` for per-monitor vsync waiting.
  /// on secondary monitors whose refresh rate differs from the primary.
  #[cfg(target_os = "windows")]
  pub fn start_workspace_switch(
    &mut self,
    windows: Vec<(Uuid, Option<WorkspaceSurrogate>, bool)>,
    order_direction: i32,
    monitor_x: i32,
    monitor_width: i32,
    monitor_y: i32,
    monitor_height: i32,
    monitor_handle: isize,
    config: &UserConfig,
  ) {
    self.workspace_switch = None;

    let ws_config = &config.value.animations.workspace_switch;

    let duration_ms = ws_config.duration_ms;

    // Slide each workspace the full monitor dimension. The outgoing workspace
    // exits the screen completely (no residual sliver at the trailing edge),
    // and the incoming workspace starts one full monitor away. The two
    // workspaces keep their normal outer-gap spacing during the slide rather
    // than being pulled together by a seam-gap reduction.
    let slide_distance_h = monitor_width.max(1);
    let slide_distance_v = monitor_height.max(1);

    let ws_windows: HashMap<Uuid, WorkspaceSwitchEntry> = windows
      .into_iter()
      .map(|(id, surrogate, is_incoming)| {
        (id, WorkspaceSwitchEntry { surrogate, is_incoming })
      })
      .collect();

    if !ws_windows.is_empty() {
      tracing::info!(
        "Starting workspace-switch animation: style={:?}, direction={:?}, \
         order={}, monitor=({monitor_x},{monitor_y},{monitor_width}x{monitor_height}), \
         slide_distance=({slide_distance_h}x{slide_distance_v}), \
         windows={}",
        ws_config.style,
        ws_config.direction,
        order_direction,
        ws_windows.len(),
      );
      // Install the per-monitor DXGI vsync waiter so the timer thread wakes
      // up right after each vblank, giving a full frame period for surrogate
      // updates before the next DWM composition.
      match DxgiVsyncWaiter::for_monitor(monitor_handle) {
        Ok(waiter) => {
          *self
            .animation_timer_vsync
            .lock()
            .expect("animation mutex poisoned") = Some(waiter);
        }
        Err(err) => {
          tracing::warn!(
            ?err,
            "failed to create vsync waiter for workspace switch"
          );
        }
      }
      self.workspace_switch = Some(WorkspaceSwitchState {
        windows: ws_windows,
        start_time: None,
        duration: Duration::from_millis(u64::from(duration_ms)),
        easing: ws_config.easing.clone(),
        style: ws_config.style.clone(),
        slide_direction: ws_config.direction.clone(),
        order_direction,
        monitor_x,
        monitor_width,
        monitor_y,
        monitor_height,
        slide_distance_h,
        slide_distance_v,
        zoom_factor: ws_config.zoom_factor.clamp(0.0, 1.0),
        vsync_anchored: false,
      });
    } else {
      tracing::warn!("Workspace-switch skipped: no windows to animate.");
    }
  }

  /// Starts an iris-wipe workspace transition driven by `overlay`.
  ///
  /// The overlay is a frozen snapshot of the outgoing workspace shown on top of
  /// the (already switched) real windows. The hole grows from radius `0` to
  /// `max_radius` — which fully covers the monitor — from `(origin_x, origin_y)`
  /// over `duration_ms`, revealing the live incoming workspace beneath. Installs
  /// the per-monitor vsync waiter so the wipe is frame-aligned, mirroring the
  /// slide driver.
  #[cfg(target_os = "windows")]
  pub fn start_iris_switch(
    &mut self,
    overlay: NativeIrisOverlay,
    origin_x: i32,
    origin_y: i32,
    max_radius: i32,
    monitor_handle: isize,
    duration_ms: u32,
    easing: EasingFunction,
  ) {
    tracing::info!(
      "Starting iris-wipe workspace switch: origin=({origin_x},{origin_y}), \
       max_radius={max_radius}, duration_ms={duration_ms}."
    );
    match DxgiVsyncWaiter::for_monitor(monitor_handle) {
      Ok(waiter) => {
        *self
          .animation_timer_vsync
          .lock()
          .expect("animation mutex poisoned") = Some(waiter);
      }
      Err(err) => {
        tracing::warn!(?err, "failed to create vsync waiter for iris switch");
      }
    }
    self.iris_switch = Some(IrisSwitchState {
      overlay,
      origin_x,
      origin_y,
      max_radius: max_radius.max(1),
      start_time: None,
      duration: Duration::from_millis(u64::from(duration_ms)),
      easing,
    });
    self.ensure_timer_running();
  }

  /// Drops any in-flight iris overlay immediately, revealing the real windows
  /// beneath.
  ///
  /// Called before snapshotting for a new switch so the snapshot captures the
  /// real current workspace rather than the previous overlay mid-wipe — making
  /// rapid switches play as clean successive wipes instead of nested ones.
  #[cfg(target_os = "windows")]
  pub fn clear_iris_switch(&mut self) {
    self.iris_switch = None;
  }

  /// Starts an open animation for a newly appearing window.
  ///
  /// The surrogate animates from a computed start state (determined by
  /// `window_open.direction`) to the window's final target rect. A
  /// `ResizeSession` handles all visuals; the real window remains cloaked
  /// until the animation completes.
  ///
  /// No-ops when `style` is `None` and `opacity_from` is `1.0` (nothing
  /// would visually change for the duration).
  #[cfg(target_os = "windows")]
  pub fn start_open_animation(
    &mut self,
    window_id: Uuid,
    target_rect: Rect,
    monitor_rect: Rect,
    effect_opacity: u8,
    corner_style: CornerStyle,
    config: &UserConfig,
    native_window: &NativeWindow,
  ) {
    let anim_config = &config.value.animations.window_open;
    let is_zoom = anim_config.style == WindowTransitionStyle::Zoom;
    let is_stationary = anim_config.style.is_stationary();

    // Skip `None` style (no slide, no zoom) with no opacity change — nothing
    // would visually change for the duration.
    if is_stationary && !is_zoom && anim_config.opacity_from >= 1.0 {
      return;
    }

    // Pace the open animation on the window's own monitor.
    self.ensure_waiter_for(DxgiVsyncWaiter::window_monitor(
      native_window.hwnd(),
    ));

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

    // For `None`/fade style only: hold at progress 0.0 so the app can paint
    // before the surrogate reveals it. At progress 0.0 the surrogate sits at
    // the window's target rect with `start_opacity`, so showing it too early
    // would flash a black (unpainted) rectangle at the window's position.
    //
    // Slide and zoom surrogates are invisible at progress 0.0 (off-screen and
    // zero-size respectively), so the grace period only adds a blank gap for
    // those styles — omit it so the animation starts immediately and the blank
    // between cloak and first visible surrogate pixel is minimised.
    if is_stationary && !is_zoom {
      anim.start_delay = OPEN_PAINT_GRACE;
    }

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
      SessionOptions {
        effect_opacity,
        initially_visible: false,
        corner_style,
        place_at_top: true,
        // No cache for open animations: the window is new, so no prior
        // sample exists.
        edge_color: None,
      },
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
  /// The surrogate is created and shown immediately as a pixel-identical
  /// overlay of the (still-visible) window; the caller cloaks the real window
  /// only after this returns, so the surrogate is already covering it and no
  /// gap exposes the desktop. The surrogate style is determined by
  /// `window_close.style`:
  /// - `None`/`Zoom`: surrogate stays at `current_rect`, fades/zooms out.
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
    corner_style: CornerStyle,
    config: &UserConfig,
    native_window: &NativeWindow,
  ) {
    if self.pending_close_windows.contains_key(&window_id) {
      return;
    }

    let anim_config = &config.value.animations.window_close;
    let is_zoom = anim_config.style == WindowTransitionStyle::Zoom;
    let is_stationary = anim_config.style.is_stationary();

    // Skip stationary style (no slide, no zoom) with no opacity change —
    // nothing would visually change for the duration.
    if is_stationary && !is_zoom && anim_config.opacity_to >= 1.0 {
      return;
    }

    // Pace the close animation on the window's own monitor.
    self.ensure_waiter_for(DxgiVsyncWaiter::window_monitor(
      native_window.hwnd(),
    ));

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

    if anim_config.opacity_to < 1.0 {
      let effect_frac = effect_opacity as f32 / 255.0;
      let target_frac = anim_config.opacity_to.clamp(0.0, 1.0) * effect_frac;
      anim.start_opacity = Some(OpacityValue(effect_frac));
      anim.target_opacity = Some(OpacityValue(target_frac));
    }

    match ResizeSession::begin(
      native_window.hwnd(),
      &current_rect,
      &target_rect,
      SessionOptions {
        effect_opacity,
        initially_visible: false,
        corner_style,
        place_at_top: false,
        // Reuse a cached color when the closing window was recently
        // animated; otherwise sample — the window is still on screen.
        edge_color: self.cached_edge_color(native_window.hwnd().0),
      },
    ) {
      Ok(mut session) => {
        // Show the surrogate immediately so it covers the window before the
        // caller cloaks it (a seamless, pixel-identical handoff).
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

