use crate::{platform_impl, Dispatcher, NativeWindow, OpacityValue, Rect};

/// Shared GPU context for animation windows.
///
/// # Platform-specific
///
/// - **macOS**: No-op; Core Animation manages GPU resources
///   automatically.
/// - **Windows**: Holds a single D3D11 device and `DirectComposition`
///   device shared across all animation windows.
pub struct AnimationContext {
  pub(crate) inner: platform_impl::AnimationContext,
}

impl AnimationContext {
  /// Creates a new shared animation context.
  pub fn new() -> crate::Result<Self> {
    let inner = platform_impl::AnimationContext::new()?;
    Ok(Self { inner })
  }

  /// Commits all pending compositor changes.
  ///
  /// Call once per tick after all `AnimationWindow::update` calls.
  pub fn commit(&self) -> crate::Result<()> {
    self.inner.commit()
  }
}

/// Per-window overlay for animating a single window transition.
///
/// # Platform-specific
///
/// - **macOS**: A transparent `NSWindow` with a single `CALayer`,
///   ordered just above the source window. Core Animation handles GPU
///   compositing.
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
  /// The overlay window itself is never repositioned; only the
  /// contained layer moves. Does not commit; call
  /// `AnimationContext::commit` after all per-tick updates.
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

impl std::fmt::Debug for AnimationWindow {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AnimationWindow").finish_non_exhaustive()
  }
}
