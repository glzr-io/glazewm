use std::{
  collections::{HashMap, HashSet},
  time::{Duration, Instant},
};

use anyhow::Context;
use tokio::sync::mpsc;
use uuid::Uuid;
use wm_common::AnimationEffectsConfig;
use wm_platform::{
  AnimationContext, AnimationWindow, Dispatcher, EasingFunction,
  OpacityValue, Rect,
};

use crate::{
  models::{NativeMonitorProperties, WindowContainer},
  traits::{CommonGetters, WindowGetters},
  user_config::UserConfig,
};

/// State of an individual window animation.
///
/// A window corresponds to a maximum of one [`WindowAnimationState`] at a
/// time.
#[derive(Clone, Debug)]
struct WindowAnimationState {
  start_time: Instant,
  duration: Duration,
  easing: EasingFunction,

  /// Target frame rate for the animation.
  frame_rate: u32,

  /// Start and target positions for the animation.
  start_rect: Rect,
  target_rect: Rect,

  /// Start and target opacity for the animation, or `None` if no opacity
  /// animation is active.
  start_opacity: Option<OpacityValue>,
  target_opacity: Option<OpacityValue>,
}

impl WindowAnimationState {
  /// Creates a new movement animation between two rects.
  fn new(
    start_rect: Rect,
    target_rect: Rect,
    config: &AnimationEffectsConfig,
    frame_rate: u32,
  ) -> Self {
    Self {
      start_time: Instant::now(),
      duration: Duration::from_millis(u64::from(config.duration_ms)),
      frame_rate,
      easing: config.easing.clone(),
      start_rect,
      target_rect,
      start_opacity: None,
      target_opacity: None,
    }
  }

  /// Returns the normalized animation progress in `[0.0, 1.0]`.
  fn progress(&self) -> f32 {
    let elapsed = self.start_time.elapsed();

    if elapsed >= self.duration {
      1.0
    } else {
      #[allow(clippy::cast_precision_loss)]
      let progress =
        elapsed.as_millis() as f32 / self.duration.as_millis() as f32;

      progress.clamp(0.0, 1.0)
    }
  }

  /// Whether the animation has completed.
  // LINT: Progress is clamped to [0.0, 1.0], so exact comparison is safe.
  #[allow(clippy::float_cmp)]
  fn is_complete(&self) -> bool {
    self.progress() == 1.0
  }

  /// Returns the interpolated rect at the current animation progress.
  fn current_rect(&self) -> Rect {
    let eased_progress = self.easing.apply(self.progress());
    self
      .start_rect
      .interpolate(&self.target_rect, eased_progress)
  }

  /// Returns the interpolated opacity at the current animation progress,
  /// or `None` if no opacity animation is active.
  fn current_opacity(&self) -> Option<OpacityValue> {
    let (start, end) =
      (self.start_opacity.as_ref()?, self.target_opacity.as_ref()?);

    let eased_progress = self.easing.apply(self.progress());
    Some(start.interpolate(end, eased_progress))
  }
}

/// Manages animations for all windows.
pub struct AnimationManager {
  /// Active animations keyed by window ID.
  animations: HashMap<Uuid, WindowAnimationState>,

  /// Sender for animation tick events.
  tick_tx: mpsc::UnboundedSender<()>,

  /// Receiver for animation tick events.
  pub tick_rx: mpsc::UnboundedReceiver<()>,

  /// Per-window overlay windows keyed by window ID.
  windows: HashMap<Uuid, AnimationWindow>,

  /// Shared GPU context for animation overlay windows. Lazily
  /// initialized on the first animation.
  context: Option<AnimationContext>,

  /// Handle to the running tick task, if any.
  tick_task: Option<tokio::task::JoinHandle<()>>,
}

impl AnimationManager {
  pub fn new() -> Self {
    let (tick_tx, tick_rx) = mpsc::unbounded_channel();

    Self {
      animations: HashMap::new(),
      tick_tx,
      tick_rx,
      windows: HashMap::new(),
      context: None,
      tick_task: None,
    }
  }

  /// Whether an animation is currently active for a given window.
  pub fn is_animating(&self, window_id: &Uuid) -> bool {
    self.animations.contains_key(window_id)
  }

  /// Gets the window IDs of animations that have completed.
  pub fn completed_ids(&self) -> HashSet<Uuid> {
    self
      .animations
      .iter()
      .filter(|(_, anim)| anim.is_complete())
      .map(|(id, _)| *id)
      .collect::<HashSet<_>>()
  }

  /// Destroys the animation window and clears animation state.
  pub fn destroy_animation(
    &mut self,
    window_id: &Uuid,
  ) -> anyhow::Result<()> {
    self.animations.remove(window_id);
    self.update_tick_rate();

    if let Some(anim_window) = self.windows.remove(window_id) {
      anim_window.destroy()?;
    }

    Ok(())
  }

  /// Updates all active animations during a single tick.
  ///
  /// Updates get batched into a single compositor transaction.
  pub fn tick_update(&mut self) -> anyhow::Result<()> {
    if self.animations.is_empty() {
      return Ok(());
    }

    self
      .context
      .as_ref()
      .context("Animation context not initialized.")?
      .transaction(|| {
        for (id, anim) in &self.animations {
          if !anim.is_complete() {
            if let Some(anim_window) = self.windows.get(id) {
              anim_window.update(
                &anim.current_rect(),
                anim.current_opacity().as_ref(),
              )?;
            }
          }
        }
        anyhow::Ok(())
      })?
  }

  /// Determines whether a new animation should be started for a window.
  pub fn should_start_animation(
    &self,
    window: &WindowContainer,
    monitor_properties: &NativeMonitorProperties,
    is_opening: bool,
    target_rect: &Rect,
    config: &UserConfig,
  ) -> bool {
    // Skip animation if:
    //  - The window is minimized.
    //  - The window is maximized (macOS only - can't override the OS's
    //    animation).
    //  - The window is hidden in the corner, but not animating. Safeguards
    //    against race condition where window finished an animation, but
    //    hasn't been moved to the real window position yet.
    if window.native_properties().is_minimized
      || (window.native_properties().is_maximized
        && cfg!(target_os = "macos"))
      || (!self.is_animating(&window.id())
        && window.is_in_corner(&monitor_properties.working_area))
    {
      return false;
    }

    match is_opening {
      true if config.value.animations.window_open.enabled => {
        !self.animations.contains_key(&window.id())
      }
      false if config.value.animations.window_move.enabled => {
        // If the window is mid-animation, compare the previous animation
        // target to the new target.
        let frame = window.native_properties().frame;
        let prev_rect = self
          .animations
          .get(&window.id())
          .map_or(&frame, |anim| &anim.target_rect);

        let distance = (prev_rect.x() - target_rect.x()).abs()
          + (prev_rect.y() - target_rect.y()).abs()
          + (prev_rect.width() - target_rect.width()).abs()
          + (prev_rect.height() - target_rect.height()).abs();

        #[allow(clippy::cast_possible_wrap)]
        let threshold_px =
          config.value.animations.window_move.threshold_px as i32;

        distance > threshold_px
      }
      _ => false,
    }
  }

  /// Starts an animation.
  #[allow(clippy::too_many_arguments)]
  pub fn start_animation(
    &mut self,
    window: &WindowContainer,
    monitor_properties: &NativeMonitorProperties,
    is_opening: bool,
    target_rect: Rect,
    config: &UserConfig,
    dispatcher: &Dispatcher,
  ) -> anyhow::Result<()> {
    let existing_animation = self.animations.get(&window.id());

    // Sync the frame rate to the monitor's refresh rate. Since ticks are
    // skipped if the animation is behind, the frame rate is variable.
    let frame_rate = monitor_properties.refresh_rate.unwrap_or(60);

    let animation = if is_opening {
      WindowAnimationState::new(
        target_rect.scale_from_center(0.9),
        target_rect,
        &config.value.animations.window_open,
        frame_rate,
      )
    } else {
      let start_rect = existing_animation.map_or_else(
        || window.native_properties().frame.clone(),
        WindowAnimationState::current_rect,
      );

      WindowAnimationState::new(
        start_rect,
        target_rect,
        &config.value.animations.window_move,
        frame_rate,
      )
    };

    self.animations.insert(window.id(), animation.clone());

    // Resize existing overlay to the new bounding box when the target
    // changes mid-flight, preserving the screenshot and z-order.
    if let Some(anim_window) = self.windows.get_mut(&window.id()) {
      anim_window
        .resize(&animation.start_rect.union(&animation.target_rect))?;
    } else {
      let context = match &self.context {
        Some(ctx) => ctx,
        None => self
          .context
          .get_or_insert(AnimationContext::new(dispatcher)?),
      };

      let anim_window = AnimationWindow::new(
        context,
        &window.native(),
        &animation.start_rect,
        &animation.start_rect.union(&animation.target_rect),
        animation.current_opacity(),
        dispatcher,
      )?;

      self.windows.insert(window.id(), anim_window);
    }

    // Start the timer after the window has been created.
    // TODO: Start times for animations will differ slightly between
    // windows within the same platform sync.
    if let Some(animation) = self.animations.get_mut(&window.id()) {
      animation.start_time = Instant::now();
    }

    self.update_tick_rate();

    Ok(())
  }

  /// Spawns a task for emitting ticks at the target frame rate.
  ///
  /// Cancels existing tick task if there is one. The ticks are emitted at
  /// the highest frame rate among the animated windows, capped by
  /// `max_frame_rate`.
  ///
  /// Called on animation start and completion.
  fn update_tick_rate(&mut self) {
    if let Some(handle) = self.tick_task.take() {
      handle.abort();
    }

    // Get the highest frame rate among the animated windows.
    let Some(frame_rate) =
      self.animations.values().map(|anim| anim.frame_rate).max()
    else {
      return;
    };

    let frame_time = Duration::from_millis(u64::from(1000 / frame_rate));
    let tick_tx = self.tick_tx.clone();

    self.tick_task = Some(tokio::spawn(async move {
      let mut interval = tokio::time::interval(frame_time);
      interval
        .set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

      loop {
        interval.tick().await;
        if tick_tx.send(()).is_err() {
          break;
        }
      }
    }));
  }
}
