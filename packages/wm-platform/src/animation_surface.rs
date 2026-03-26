use crate::{platform_impl, Dispatcher, NativeWindow, OpacityValue, Rect};

/// Identifier for a layer within an `AnimationSurface`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LayerId(pub(crate) u64);

/// A collection of screenshot layers for animating window transitions.
///
/// # Platform-specific
///
/// - **macOS**: A single transparent `NSWindow` with `CALayer` sublayers.
///   Core Animation handles GPU compositing.
/// - **Windows**: One layered overlay `HWND` per animation layer, using
///   `UpdateLayeredWindow` for rendering.
pub struct AnimationSurface {
  inner: platform_impl::AnimationSurface,
}

impl AnimationSurface {
  /// Creates a new `AnimationSurface`.
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let inner = platform_impl::AnimationSurface::new(dispatcher)?;
    Ok(Self { inner })
  }

  /// Screenshots the target window and adds a layer.
  ///
  /// Returns a `LayerId` handle for future updates and removal.
  pub fn add_layer(
    &mut self,
    window: &NativeWindow,
    rect: &Rect,
    opacity: Option<f32>,
  ) -> crate::Result<LayerId> {
    self.inner.add_layer(window, rect, opacity)
  }

  /// Updates frame and opacity for active layers.
  ///
  /// Implicit animations are disabled so updates take effect immediately.
  pub fn update_layers(
    &self,
    updates: Vec<(LayerId, Rect, Option<OpacityValue>)>,
  ) -> crate::Result<()> {
    self.inner.update_layers(updates)
  }

  /// Removes a layer from the surface.
  pub fn remove_layer(&mut self, id: LayerId) -> crate::Result<()> {
    self.inner.remove_layer(id)
  }

  /// Returns whether the surface has any active layers.
  pub fn has_layers(&self) -> crate::Result<bool> {
    self.inner.has_layers()
  }

  /// Hides the surface without destroying it.
  ///
  /// The surface can be shown again via `show`, avoiding the cost of
  /// recreating platform resources.
  pub fn hide(&self) -> crate::Result<()> {
    self.inner.hide()
  }

  /// Shows a previously hidden surface.
  pub fn show(&self) -> crate::Result<()> {
    self.inner.show()
  }

  /// Destroys the surface and all layers.
  pub fn destroy(self) -> crate::Result<()> {
    self.inner.destroy()
  }
}

impl std::fmt::Debug for AnimationSurface {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AnimationSurface").finish_non_exhaustive()
  }
}
