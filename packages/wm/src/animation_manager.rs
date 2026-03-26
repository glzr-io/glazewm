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
  AnimationSurface, Dispatcher, EasingFunction, LayerId, NativeWindow,
  OpacityValue, Rect,
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

  /// Shared animation surface with one layer per animating window.
  surface: Option<AnimationSurface>,

  /// Maps window UUIDs to their layer handles within the surface.
  layer_ids: HashMap<Uuid, LayerId>,
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

  /// Creates or replaces the animation layer for a window.
  ///
  /// Reuses the existing `AnimationSurface` if available, showing it
  /// again if it was hidden. Only creates a new surface on first use.
  fn create_layer(
    &mut self,
    window_id: Uuid,
    native_window: &NativeWindow,
    initial_rect: &Rect,
    opacity: Option<f32>,
    dispatcher: &Dispatcher,
  ) {
    // Remove any existing layer for this window.
    let _ = self.destroy_layer(&window_id);

    // Reuse the existing surface, or create one on first use.
    if let Some(surface) = &self.surface {
      // Surface exists but may be hidden — show it.
      if let Err(err) = surface.show() {
        tracing::warn!("Failed to show animation surface: {}", err);
      }
    } else {
      match AnimationSurface::new(dispatcher) {
        Ok(surface) => self.surface = Some(surface),
        Err(err) => {
          tracing::warn!("Failed to create animation surface: {}", err);
          return;
        }
      }
    }

    let surface = self.surface.as_mut().expect("surface must exist");

    match surface.add_layer(native_window, initial_rect, opacity) {
      Ok(layer_id) => {
        self.layer_ids.insert(window_id, layer_id);
      }
      Err(err) => {
        tracing::warn!("Failed to add animation layer: {}", err);
      }
    }
  }

  /// Removes the layer for a window.
  pub fn destroy_layer(&mut self, window_id: &Uuid) -> anyhow::Result<()> {
    if let Some(layer_id) = self.layer_ids.remove(window_id) {
      if let Some(surface) = &mut self.surface {
        surface.remove_layer(layer_id)?;
      }
    }

    // Hide the shared surface when no layers remain. The surface is kept
    // alive for reuse.
    if self.layer_ids.is_empty() {
      if let Some(surface) = &self.surface {
        surface.hide()?;
      }
    }

    Ok(())
  }

  /// Updates all active animations and returns the IDs of any that
  /// completed during this tick.
  pub fn update(&mut self) -> anyhow::Result<Vec<Uuid>> {
    if self.animations.is_empty() {
      return Ok(Vec::new());
    }

    // Update layer positions for in-progress animations.
    let updates: Vec<_> = self
      .animations
      .iter()
      .filter(|(_, anim)| !anim.is_complete())
      .filter_map(|(id, anim)| {
        let layer_id = self.layer_ids.get(id)?;
        Some((*layer_id, anim.current_rect(), anim.current_opacity()))
      })
      .collect();

    if let Some(surface) = &self.surface {
      surface.update_layers(updates)?;
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
  ) {
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

    self.create_layer(
      window_id,
      native_window,
      &animation.current_rect(),
      animation.current_opacity().map(|opacity| opacity.0),
      dispatcher,
    );

    // Start the timer after the layer has been created.
    // TODO: Start times for animations will differ slightly between
    // windows within the same platform sync.
    if let Some(animation) = self.animations.get_mut(&window_id) {
      animation.start_time = Instant::now();
    }
  }
}
