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
use wm_common::AnimationEffectsConfig;
use wm_platform::{
  AnimationWindow, Dispatcher, EasingFunction, NativeWindow, OpacityValue,
  Rect,
};

use crate::{
  models::NativeWindowProperties, traits::CommonGetters,
  user_config::UserConfig,
};

/// State of an individual window animation.
///
/// A window corresponds to a maximum of one [`WindowAnimationState`] at a
/// time.
#[derive(Clone, Debug)]
pub struct WindowAnimationState {
  pub start_time: Instant,
  pub duration: Duration,
  pub easing: EasingFunction,

  /// Start and target positions for the animation.
  pub start_rect: Rect,
  pub target_rect: Rect,

  /// Start and target opacity for the animation, or `None` if no opacity
  /// animation is active.
  pub start_opacity: Option<OpacityValue>,
  pub target_opacity: Option<OpacityValue>,
}

impl WindowAnimationState {
  /// Creates a new movement animation between two rects.
  pub fn new(
    start_rect: Rect,
    target_rect: Rect,
    config: &AnimationEffectsConfig,
  ) -> Self {
    Self {
      start_time: Instant::now(),
      duration: Duration::from_millis(u64::from(config.duration_ms)),
      easing: config.easing.clone(),
      start_rect,
      target_rect,
      start_opacity: None,
      target_opacity: None,
    }
  }

  /// Returns the normalized animation progress in `[0.0, 1.0]`.
  pub fn progress(&self) -> f32 {
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
  pub fn is_complete(&self) -> bool {
    self.progress() == 1.0
  }

  /// Returns the interpolated rect at the current animation progress.
  pub fn current_rect(&self) -> Rect {
    let eased_progress = self.easing.apply(self.progress());
    self
      .start_rect
      .interpolate(&self.target_rect, eased_progress)
  }

  /// Returns the interpolated opacity at the current animation progress,
  /// or `None` if no opacity animation is active.
  pub fn current_opacity(&self) -> Option<OpacityValue> {
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
  animation_tick_tx: mpsc::UnboundedSender<()>,

  /// Whether the animation timer is currently running.
  animation_timer_running: Arc<AtomicBool>,

  /// Per-window overlay windows keyed by window ID.
  windows: HashMap<Uuid, AnimationWindow>,
}

impl AnimationManager {
  pub fn new(animation_tick_tx: mpsc::UnboundedSender<()>) -> Self {
    Self {
      animations: HashMap::new(),
      animation_tick_tx,
      animation_timer_running: Arc::new(AtomicBool::new(false)),
      windows: HashMap::new(),
    }
  }

  /// Removes a window's animation state.
  pub fn remove_animation(&mut self, window_id: &Uuid) {
    self.animations.remove(window_id);
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
      // TODO: The focused monitor may not be the one with the animation.
      .and_then(|c| CommonGetters::monitor(&c))
      .and_then(|m| m.native_properties().refresh_rate)
      .unwrap_or(config.value.animations.max_frame_rate);

    let frame_rate =
      refresh_rate.min(config.value.animations.max_frame_rate);

    // Convert refresh rate to milliseconds per frame.
    let frame_time_ms = 1000 / frame_rate;

    // TODO: The spawned interval always ticks, even if an animation is not
    // active.
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

  /// Destroys the animation window for a given window ID.
  pub fn destroy_animation_window(
    &mut self,
    window_id: &Uuid,
  ) -> anyhow::Result<()> {
    if let Some(anim_window) = self.windows.remove(window_id) {
      anim_window.destroy()?;
    }

    Ok(())
  }

  /// Updates all active animations and returns the IDs of any that
  /// completed during this tick.
  pub fn update(&mut self) -> anyhow::Result<Vec<Uuid>> {
    if self.animations.is_empty() {
      return Ok(Vec::new());
    }

    for (id, anim) in &self.animations {
      if !anim.is_complete() {
        if let Some(anim_window) = self.windows.get(id) {
          anim_window
            .update(&anim.current_rect(), anim.current_opacity())?;
        }
      }
    }

    // Return IDs of completed animations. Removal is deferred so that
    // `should_start_animation` can still read their `target_rect` during
    // the final redraw.
    Ok(
      self
        .animations
        .iter()
        .filter(|(_, anim)| anim.is_complete())
        .map(|(id, _)| *id)
        .collect::<Vec<_>>(),
    )
  }

  /// Determines whether a new animation should be started for a window.
  pub fn should_start_animation(
    &self,
    window_id: &Uuid,
    is_opening: bool,
    target_rect: &Rect,
    window_properties: &NativeWindowProperties,
    config: &UserConfig,
  ) -> bool {
    if window_properties.is_minimized {
      return false;
    }

    match is_opening {
      true if config.value.animations.window_open.enabled => {
        !self.animations.contains_key(window_id)
      }
      false if config.value.animations.window_move.enabled => {
        // If the window is mid-animation, compare the previous animation
        // target to the new target.
        let prev_rect = self
          .animations
          .get(window_id)
          .map_or(&window_properties.frame, |anim| &anim.target_rect);

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
    window_id: Uuid,
    is_opening: bool,
    target_rect: Rect,
    native_window: &NativeWindow,
    window_properties: &NativeWindowProperties,
    config: &UserConfig,
    dispatcher: &Dispatcher,
  ) -> anyhow::Result<()> {
    let existing_animation = self.animations.get(&window_id);

    let animation = if is_opening {
      WindowAnimationState::new(
        target_rect.scale_from_center(0.9),
        target_rect,
        &config.value.animations.window_open,
      )
    } else {
      let start_rect = existing_animation.map_or_else(
        || window_properties.frame.clone(),
        WindowAnimationState::current_rect,
      );

      WindowAnimationState::new(
        start_rect,
        target_rect,
        &config.value.animations.window_move,
      )
    };

    self.animations.insert(window_id, animation.clone());

    // Resize existing overlay to the new bounding box when the target
    // changes mid-flight, preserving the screenshot and z-order.
    if let Some(anim_window) = self.windows.get_mut(&window_id) {
      anim_window.resize(&animation.start_rect, &animation.target_rect)?;
    } else {
      let anim_window = AnimationWindow::new(
        dispatcher,
        native_window,
        &animation.start_rect,
        &animation.target_rect,
        animation.current_opacity().map(|o| o.0),
      )?;

      self.windows.insert(window_id, anim_window);
    }

    // Start the timer after the window has been created.
    // TODO: Start times for animations will differ slightly between
    // windows within the same platform sync.
    if let Some(animation) = self.animations.get_mut(&window_id) {
      animation.start_time = Instant::now();
    }

    Ok(())
  }
}
