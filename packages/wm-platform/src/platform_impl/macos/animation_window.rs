use objc2::{
  rc::Retained, runtime::AnyObject, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSScreen, NSWindow, NSWindowOrderingMode,
  NSWindowStyleMask,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGImage;
use objc2_quartz_core::{CALayer, CATransaction};

use crate::{
  platform_impl::CGRectExt, Dispatcher, NativeWindow,
  NativeWindowExtMacOs, OpacityValue, Rect, ThreadBound,
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
  /// Overlay bounds in CG (screen) coordinates.
  outer_bounds: CGRect,
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
    let cg_image = window.screen_capture()?;
    let outer_bounds = CGRect::from(outer_rect.clone());

    let (ns_window, layer) = dispatcher.dispatch_sync(|| {
      // SAFETY: `dispatch_sync` runs on the main thread.
      let mtm = unsafe { MainThreadMarker::new_unchecked() };

      let screens = NSScreen::screens(mtm);
      let primary_screen = screens.iter().next();
      let primary_height = primary_screen
        .as_ref()
        .map_or(0.0, |s| s.frame().size.height);
      let scale_factor = primary_screen
        .as_ref()
        .map_or(2.0, |s| s.backingScaleFactor());

      let ns_window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
          NSWindow::alloc(mtm),
          outer_bounds.to_ns_rect(primary_height),
          NSWindowStyleMask::Borderless,
          NSBackingStoreType::Buffered,
          false,
        )
      };

      ns_window.setBackgroundColor(Some(&NSColor::clearColor()));
      ns_window.setOpaque(false);
      ns_window.setIgnoresMouseEvents(true);
      // SAFETY: We manage lifetime via `ThreadBound` + `orderOut`.
      unsafe { ns_window.setReleasedWhenClosed(false) };

      let content_view = ns_window
        .contentView()
        .expect("NSWindow must have a content view");
      content_view.setWantsLayer(true);

      let root_layer = content_view
        .layer()
        .expect("layer must exist after setWantsLayer");

      let layer = CALayer::new();

      // SAFETY: `CGImageRef` is accepted by `CALayer.contents`.
      unsafe {
        layer.setContents(Some(
          &*std::ptr::from_ref::<CGImage>(&cg_image).cast::<AnyObject>(),
        ));
      };

      #[allow(clippy::cast_precision_loss)]
      let image_size = CGSize {
        width: CGImage::width(Some(&cg_image)) as f64 / scale_factor,
        height: CGImage::height(Some(&cg_image)) as f64 / scale_factor,
      };

      CATransaction::begin();
      CATransaction::setDisableActions(true);
      layer.setFrame(Self::layer_frame(
        inner_rect,
        &outer_bounds,
        image_size,
      ));

      if let Some(opacity) = opacity {
        layer.setOpacity(opacity.0);
      }

      root_layer.addSublayer(&layer);
      CATransaction::commit();

      #[allow(clippy::cast_possible_wrap)]
      ns_window.orderWindow_relativeTo(
        NSWindowOrderingMode::Above,
        window.id().0 as isize,
      );

      (
        ThreadBound::new(ns_window, dispatcher.clone()),
        ThreadBound::new(layer, dispatcher.clone()),
      )
    })?;

    Ok(Self {
      ns_window,
      layer,
      outer_bounds,
    })
  }

  /// Converts `inner_rect` from CG screen coordinates to layer-local
  /// coordinates (bottom-left origin, y-up).
  fn layer_frame(
    inner_rect: &Rect,
    outer_bounds: &CGRect,
    size: CGSize,
  ) -> CGRect {
    CGRect::new(
      CGPoint {
        x: f64::from(inner_rect.x()) - outer_bounds.origin.x,
        y: outer_bounds.size.height
          - (f64::from(inner_rect.y()) - outer_bounds.origin.y)
          - size.height,
      },
      size,
    )
  }

  /// Resizes the `NSWindow` to cover the union of `start_rect` and
  /// `target_rect`, updating the stored CG origin.
  ///
  /// Called when an animation's target changes mid-flight so the
  /// existing screenshot and z-order are preserved.
  pub(crate) fn resize(&mut self, outer_rect: &Rect) -> crate::Result<()> {
    self.outer_bounds = CGRect::from(outer_rect.clone());

    self.ns_window.with(|ns_window| {
      // SAFETY: `with` runs on the main thread.
      let mtm = unsafe { MainThreadMarker::new_unchecked() };

      let primary_height = NSScreen::screens(mtm)
        .iter()
        .next()
        .map_or(0.0, |s| s.frame().size.height);

      ns_window.setFrame_display(
        self.outer_bounds.to_ns_rect(primary_height),
        false,
      );
    })
  }

  /// Repositions the layer within the window to the given rect and
  /// updates opacity.
  ///
  /// The `NSWindow` frame is never changed; only the `CALayer` moves.
  pub(crate) fn update(
    &self,
    inner_rect: &Rect,
    opacity: Option<OpacityValue>,
  ) -> crate::Result<()> {
    let size = CGSize {
      width: f64::from(inner_rect.width()),
      height: f64::from(inner_rect.height()),
    };
    let frame = Self::layer_frame(inner_rect, &self.outer_bounds, size);

    self.layer.with(move |layer| {
      layer.setFrame(frame);

      if let Some(opacity) = opacity {
        layer.setOpacity(opacity.0);
      }
    })
  }

  /// Removes the overlay window from the screen.
  pub(crate) fn destroy(self) -> crate::Result<()> {
    // TODO: Actually destroy the window.
    self.ns_window.with(|ns_window| ns_window.orderOut(None))
  }
}
