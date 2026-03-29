use crate::{platform_impl, Dispatcher, NativeWindow, OpacityValue, Rect};

/// Shared context for animation windows.
///
/// Allows for batching updates across animation windows. Internally, this
/// holds GPU resources that can be shared between animation window
/// instances.
pub struct AnimationContext {
  inner: platform_impl::AnimationContext,
}

impl AnimationContext {
  /// Creates a new shared animation context.
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let inner = platform_impl::AnimationContext::new(dispatcher)?;
    Ok(Self { inner })
  }

  /// Executes `update_fn` inside a compositor transaction, committing all
  /// pending changes once `update_fn` returns.
  ///
  /// On macOS, `update_fn` runs inside a single `CATransaction` on the
  /// main thread. On Windows, `update_fn` runs followed by a
  /// `DirectComposition` commit.
  pub fn transaction<F, R>(&self, update_fn: F) -> crate::Result<R>
  where
    F: FnOnce() -> R + Send,
    R: Send,
  {
    self.inner.transaction(update_fn)
  }
}

/// Per-window overlay for animating a single window transition.
///
/// # Platform-specific
///
/// - **macOS**: A transparent `NSWindow` with a single `CALayer`, ordered
///   just above the source window. Core Animation handles GPU compositing.
/// - **Windows**: A layered overlay `HWND` with a `IDCompositionVisual`,
///   using `DirectComposition` for GPU compositing and
///   `Windows.Graphics.Capture` for screenshots.
pub struct AnimationWindow {
  inner: platform_impl::AnimationWindow,
}

impl AnimationWindow {
  /// Creates a new `AnimationWindow` for a single window animation.
  ///
  /// Captures a screenshot of `window` and creates an overlay spanning
  /// the union of `start_rect` and `target_rect`, ordered just above
  /// the source window.
  pub fn new(
    context: &AnimationContext,
    dispatcher: &Dispatcher,
    window: &NativeWindow,
    start_rect: &Rect,
    target_rect: &Rect,
    opacity: Option<f32>,
  ) -> crate::Result<Self> {
    let inner = platform_impl::AnimationWindow::new(
      &context.inner,
      dispatcher,
      window,
      start_rect,
      target_rect,
      opacity,
    )?;
    Ok(Self { inner })
  }

  /// Resizes the overlay to cover the union of `start_rect` and
  /// `target_rect`.
  ///
  /// Preserves the existing screenshot and z-order. Used when an
  /// animation's target changes mid-flight.
  pub fn resize(
    &mut self,
    start_rect: &Rect,
    target_rect: &Rect,
  ) -> crate::Result<()> {
    self.inner.resize(start_rect, target_rect)
  }

  /// Updates the layer position and opacity within the overlay.
  ///
  /// Does not commit; should be called within
  /// `AnimationContext::transaction` for the change to take effect.
  pub fn update(
    &self,
    rect: &Rect,
    opacity: Option<OpacityValue>,
  ) -> crate::Result<()> {
    self.inner.update(rect, opacity)
  }

  /// Destroys the overlay window and releases GPU resources.
  pub fn destroy(self) -> crate::Result<()> {
    self.inner.destroy()
  }
}
