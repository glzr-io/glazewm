use objc2::{
  rc::Retained, runtime::AnyObject, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSWindow, NSWindowOrderingMode,
  NSWindowStyleMask,
};
use objc2_core_graphics::CGImage;
use objc2_quartz_core::{CALayer, CATransaction};

use crate::{
  Dispatcher, NativeWindow, NativeWindowExtMacOs, OpacityValue, Rect,
  ThreadBound,
};

/// Shared animation context for macOS.
///
/// Core Animation manages GPU resources automatically, so no shared
/// device context is needed. The context provides `transaction` to batch
/// multiple `CALayer` updates into a single `CATransaction` on the main
/// thread.
pub(crate) struct AnimationContext {
  dispatcher: Dispatcher,
}

impl AnimationContext {
  /// Creates a shared animation context.
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    Ok(Self {
      dispatcher: dispatcher.clone(),
    })
  }

  /// Executes `update_fn` inside a single `CATransaction` on the main
  /// thread.
  ///
  /// All `CALayer` mutations performed by `update_fn` are batched into one
  /// implicit-animation-free commit.
  pub(crate) fn transaction<F, R>(&self, update_fn: F) -> crate::Result<R>
  where
    F: FnOnce() -> R + Send,
    R: Send,
  {
    self.dispatcher.dispatch_sync(|| {
      CATransaction::begin();
      CATransaction::setDisableActions(true);
      let result = update_fn();
      CATransaction::commit();
      result
    })
  }
}

/// Per-window overlay for animating a single window transition.
///
/// Each `AnimationWindow` creates its own transparent `NSWindow` sized to
/// the bounding box of the animation's start and target rects. The window
/// is ordered just above the source window via
/// `orderWindow_relativeTo`, preserving z-order among non-animated
/// windows.
///
/// The contained `CALayer` is repositioned each tick; the `NSWindow`
/// frame stays fixed for the lifetime of the animation.
pub(crate) struct AnimationWindow {
  ns_window: ThreadBound<Retained<NSWindow>>,
  layer: ThreadBound<Retained<CALayer>>,

  /// Frame of the `AnimationWindow` (in CG coordinates).
  outer_rect: Rect,

  /// Height of the primary display.
  display_height: i32,
}

impl AnimationWindow {
  /// Creates a transparent `NSWindow` covering the union of `start_rect`
  /// and `target_rect`, captures a screenshot of `window`, and orders the
  /// overlay just above the source window.
  pub(crate) fn new(
    _context: &AnimationContext,
    window: &NativeWindow,
    inner_rect: &Rect,
    outer_rect: &Rect,
    opacity: Option<OpacityValue>,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    type DispatchResult = crate::Result<(
      ThreadBound<Retained<NSWindow>>,
      ThreadBound<Retained<CALayer>>,
      i32,
    )>;

    let cg_image = window.screen_capture()?;

    let (ns_window, layer, display_height) =
      dispatcher.dispatch_sync(|| -> DispatchResult {
        // SAFETY: `Dispatcher::dispatch_sync` runs on the main thread.
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        // Get height and scale factor of the primary display.
        let (display_height, scale_factor) = {
          let display = dispatcher.primary_display()?;
          (display.bounds()?.height(), display.scale_factor()?)
        };

        let ns_window = unsafe {
          NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            // `NSWindow` expects AppKit coordinates (bottom-left origin).
            outer_rect.flip_y(display_height).into(),
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
          )
        };

        ns_window.setBackgroundColor(Some(&NSColor::clearColor()));
        ns_window.setOpaque(false);
        ns_window.setIgnoresMouseEvents(true);

        // SAFETY: `NSWindow` is normally released on close, but when the
        // `Retained<NSWindow>` field is dropped, it sends another
        // `release` which then segfaults due to double-free.
        unsafe { ns_window.setReleasedWhenClosed(false) };

        let content_view =
          ns_window.contentView().ok_or(crate::Error::Platform(
            "NSWindow must have a content view.".to_string(),
          ))?;

        content_view.setWantsLayer(true);

        let root_layer =
          content_view.layer().ok_or(crate::Error::Platform(
            "Layer must exist after `setWantsLayer`.".to_string(),
          ))?;

        // The root layer fills the content view, so a sublayer is needed
        // to animate within it.
        let layer = CALayer::new();

        // SAFETY: `CGImageRef` is accepted by `CALayer::contents`.
        unsafe {
          layer.setContents(Some(
            &*std::ptr::from_ref::<CGImage>(&cg_image).cast::<AnyObject>(),
          ));
        };

        #[allow(clippy::cast_precision_loss)]
        let image_width =
          CGImage::width(Some(&cg_image)) as f32 / scale_factor;
        #[allow(clippy::cast_precision_loss)]
        let image_height =
          CGImage::height(Some(&cg_image)) as f32 / scale_factor;

        CATransaction::begin();
        CATransaction::setDisableActions(true);

        #[allow(clippy::cast_possible_truncation)]
        let inner_rect = Rect::from_xy(
          inner_rect.x(),
          inner_rect.y(),
          image_width as i32,
          image_height as i32,
        );

        Self::update_layer(
          &layer,
          &inner_rect,
          outer_rect,
          opacity.as_ref(),
        );
        CATransaction::commit();

        root_layer.addSublayer(&layer);

        #[allow(clippy::cast_possible_wrap)]
        ns_window.orderWindow_relativeTo(
          NSWindowOrderingMode::Above,
          window.id().0 as isize,
        );

        Ok((
          ThreadBound::new(ns_window, dispatcher.clone()),
          ThreadBound::new(layer, dispatcher.clone()),
          display_height,
        ))
      })??;

    Ok(Self {
      ns_window,
      layer,
      display_height,
      outer_rect: outer_rect.clone(),
    })
  }

  /// Resizes the `NSWindow` to cover the union of `start_rect` and
  /// `target_rect`, updating the stored CG origin.
  ///
  /// Called when an animation's target changes mid-flight so the
  /// existing screenshot and z-order are preserved.
  pub(crate) fn resize(&mut self, outer_rect: &Rect) -> crate::Result<()> {
    self.outer_rect = outer_rect.clone();

    self.ns_window.with(|ns_window| {
      ns_window.setFrame_display(
        self.outer_rect.flip_y(self.display_height).into(),
        false,
      );
    })
  }

  /// Implements [`AnimationWindow::update`].
  pub(crate) fn update(
    &self,
    inner_rect: &Rect,
    opacity: Option<&OpacityValue>,
  ) -> crate::Result<()> {
    self.layer.with(|layer| {
      Self::update_layer(layer, inner_rect, &self.outer_rect, opacity);
    })
  }

  /// Removes the overlay window from the screen.
  pub(crate) fn destroy(self) -> crate::Result<()> {
    self.ns_window.with(|ns_window| ns_window.close())
  }

  /// Repositions the layer within the window to the given rect and
  /// updates opacity.
  ///
  /// The `NSWindow` frame is never changed; only the `CALayer` moves.
  fn update_layer(
    layer: &Retained<CALayer>,
    inner_rect: &Rect,
    outer_rect: &Rect,
    opacity: Option<&OpacityValue>,
  ) {
    // `inner_rect` needs to be positioned relative to `outer_rect`.
    let offset_rect = Rect::from_xy(
      inner_rect.x() - outer_rect.x(),
      inner_rect.y() - outer_rect.y(),
      inner_rect.width(),
      inner_rect.height(),
    );

    // `setFrame` expects AppKit coordinates (bottom-left origin).
    layer.setFrame(offset_rect.flip_y(outer_rect.height()).into());

    if let Some(opacity) = opacity {
      layer.setOpacity(opacity.0);
    }
  }
}
