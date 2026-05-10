use crate::{platform_impl, Dispatcher, NativeWindow, OpacityValue, Rect};

/// Shared context used by [`AnimationWindow`] instances. Holds GPU
/// resources that can be shared between animations.
///
/// Exposes a [`AnimationContext::transaction`] method for batching updates
/// across animation windows.
pub struct AnimationContext {
  inner: platform_impl::AnimationContext,
}

impl AnimationContext {
  /// Creates a new [`AnimationContext`].
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let inner = platform_impl::AnimationContext::new(dispatcher)?;
    Ok(Self { inner })
  }

  /// Executes `update_fn` inside a compositor transaction.
  ///
  /// Used with [`AnimationWindow::update`] to commit all updates together
  /// when `update_fn` returns.
  pub fn transaction<F, R>(
    &self,
    update_fn: F,
    dispatcher: &Dispatcher,
  ) -> crate::Result<R>
  where
    F: FnOnce() -> R + Send,
    R: Send,
  {
    self.inner.transaction(update_fn, dispatcher)
  }
}

/// A screenshot of a [`NativeWindow`] that can be animated performantly.
///
/// # Example usage
///
///   1. Swap in the `AnimationWindow` with the `NativeWindow`,
///   2. Perform animation.
///   3. Swap out the `AnimationWindow`.
///
/// ```no_run,compile_fail
/// let frame = real_window.frame()?;
/// let anim_window = AnimationWindow::new(context, real_window, frame, /* .. */)?;
///
/// # Hide the real window at the animation end position.
/// real_window.set_frame(frame.translate_in_direction(Direction::Left, 100))?;
/// real_window.set_transparency(&OpacityValue::from_alpha(0))?;
///
/// for i in 1..100 {
///   context.transaction(|| {
///     anim_window.update(frame.translate_in_direction(Direction::Left, i), None))
///   })??;
/// }
///
/// real_window.set_transparency(&OpacityValue::from_alpha(u8::MAX));
/// anim_window.destroy()?;
/// ```
pub struct AnimationWindow {
  inner: platform_impl::AnimationWindow,
}

impl AnimationWindow {
  /// Creates a new [`AnimationWindow`].
  ///
  /// The `outer_rect` should span the bounds of the start and end
  /// rects of the animation.
  pub fn new(
    context: &AnimationContext,
    window: &NativeWindow,
    inner_rect: &Rect,
    outer_rect: &Rect,
    opacity: Option<OpacityValue>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let inner = platform_impl::AnimationWindow::new(
      &context.inner,
      window,
      inner_rect,
      outer_rect,
      opacity,
      dispatcher,
    )?;

    Ok(Self { inner })
  }

  /// Resizes the window.
  ///
  /// Called when an animation's target rect changes mid-flight.
  pub fn resize(&mut self, outer_rect: &Rect) -> crate::Result<()> {
    self.inner.resize(outer_rect)
  }

  /// Updates the layer position and opacity within the window.
  ///
  /// Does not commit; should be called within
  /// `AnimationContext::transaction` for the change to take effect.
  pub fn update(
    &self,
    inner_rect: &Rect,
    opacity: Option<&OpacityValue>,
  ) -> crate::Result<()> {
    self.inner.update(inner_rect, opacity)
  }

  /// Destroys the window and releases GPU resources.
  pub fn destroy(self) -> crate::Result<()> {
    self.inner.destroy()
  }
}
