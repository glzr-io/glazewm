use objc2::{
  rc::Retained, runtime::AnyObject, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSScreen, NSWindow,
  NSWindowOrderingMode, NSWindowStyleMask,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGImage;
use objc2_foundation::NSRect;
use objc2_quartz_core::{CALayer, CATransaction};

use crate::{
  Dispatcher, NativeWindow, NativeWindowExtMacOs, OpacityValue, Rect,
  ThreadBound,
};

/// Shared animation context (no-op on macOS).
///
/// On macOS, Core Animation manages GPU resources automatically, so no
/// shared device context is needed.
pub(crate) struct AnimationContext;

impl AnimationContext {
  /// Creates a shared animation context (no-op on macOS).
  pub(crate) fn new() -> crate::Result<Self> {
    Ok(Self)
  }

  /// Commits pending changes (no-op on macOS).
  pub(crate) fn commit(&self) -> crate::Result<()> {
    Ok(())
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
  /// Top-left of the animation window in CG (screen) coordinates.
  cg_origin_x: f64,
  cg_origin_y: f64,
}

impl AnimationWindow {
  /// Creates a transparent `NSWindow` covering the union of `start_rect`
  /// and `target_rect`, captures a screenshot of `window`, and orders the
  /// overlay just above the source window.
  pub(crate) fn new(
    _context: &AnimationContext,
    dispatcher: &Dispatcher,
    window: &NativeWindow,
    start_rect: &Rect,
    target_rect: &Rect,
    opacity: Option<f32>,
  ) -> crate::Result<Self> {
    let cg_image = window.screen_capture()?;
    let source_window_number = window.id().0;
    let bounds = start_rect.union(target_rect);
    let start_rect = start_rect.clone();

    let dispatcher_clone = dispatcher.clone();

    let (ns_window, layer, cg_origin_x, cg_origin_y) =
      dispatcher.dispatch_sync(move || {
        // SAFETY: `dispatch_sync` runs on the main thread.
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        let screens = NSScreen::screens(mtm);
        let primary_height =
          screens.iter().next().map_or(0.0, |s| s.frame().size.height);

        let scale_factor = screens
          .iter()
          .next()
          .map_or(2.0, |s| s.backingScaleFactor());

        let cg_origin_x = f64::from(bounds.x());
        let cg_origin_y = f64::from(bounds.y());

        // Convert CG coordinates (top-left origin) to AppKit (bottom-left).
        let appkit_y =
          primary_height - f64::from(bounds.y()) - f64::from(bounds.height());

        let ns_rect = NSRect::new(
          objc2_foundation::NSPoint {
            x: f64::from(bounds.x()),
            y: appkit_y,
          },
          objc2_foundation::NSSize {
            width: f64::from(bounds.width()),
            height: f64::from(bounds.height()),
          },
        );

        let ns_window = unsafe {
          NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            ns_rect,
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

        if let Some(content_view) = ns_window.contentView() {
          content_view.setWantsLayer(true);
        }

        let root_layer = ns_window
          .contentView()
          .and_then(|v| v.layer())
          .expect("layer must exist after setWantsLayer");

        // Flip y-axis so sublayer origins use CG screen coordinates.
        root_layer.setGeometryFlipped(true);

        let layer = CALayer::new();

        // SAFETY: `CGImageRef` is accepted by `CALayer.contents`.
        unsafe {
          layer.setContents(Some(
            &*std::ptr::from_ref::<CGImage>(&cg_image)
              .cast::<AnyObject>(),
          ));
        };

        #[allow(clippy::cast_precision_loss)]
        let image_width =
          CGImage::width(Some(&cg_image)) as f64 / scale_factor;
        #[allow(clippy::cast_precision_loss)]
        let image_height =
          CGImage::height(Some(&cg_image)) as f64 / scale_factor;

        let frame = CGRect::new(
          CGPoint {
            x: f64::from(start_rect.x()) - cg_origin_x,
            y: f64::from(start_rect.y()) - cg_origin_y,
          },
          CGSize {
            width: image_width,
            height: image_height,
          },
        );

        CATransaction::begin();
        CATransaction::setDisableActions(true);
        layer.setFrame(frame);

        if let Some(alpha) = opacity {
          layer.setOpacity(alpha);
        }

        root_layer.addSublayer(&layer);
        CATransaction::commit();

        // Order just above the source window.
        #[allow(clippy::cast_possible_wrap)]
        ns_window.orderWindow_relativeTo(
          NSWindowOrderingMode::Above,
          source_window_number as isize,
        );

        (
          ThreadBound::new(ns_window, dispatcher_clone.clone()),
          ThreadBound::new(layer, dispatcher_clone),
          cg_origin_x,
          cg_origin_y,
        )
      })?;

    Ok(Self {
      ns_window,
      layer,
      cg_origin_x,
      cg_origin_y,
    })
  }

  /// Resizes the `NSWindow` to cover the union of `start_rect` and
  /// `target_rect`, updating the stored CG origin.
  ///
  /// Called when an animation's target changes mid-flight so the
  /// existing screenshot and z-order are preserved.
  pub(crate) fn resize(
    &mut self,
    start_rect: &Rect,
    target_rect: &Rect,
  ) -> crate::Result<()> {
    let bounds = start_rect.union(target_rect);

    self.cg_origin_x = f64::from(bounds.x());
    self.cg_origin_y = f64::from(bounds.y());

    let bounds_clone = bounds.clone();

    self.ns_window.with(move |w| {
      // SAFETY: `with` runs on the main thread.
      let mtm = unsafe { MainThreadMarker::new_unchecked() };

      let primary_height = NSScreen::screens(mtm)
        .iter()
        .next()
        .map_or(0.0, |s| s.frame().size.height);

      let appkit_y = primary_height
        - f64::from(bounds_clone.y())
        - f64::from(bounds_clone.height());

      let ns_rect = NSRect::new(
        objc2_foundation::NSPoint {
          x: f64::from(bounds_clone.x()),
          y: appkit_y,
        },
        objc2_foundation::NSSize {
          width: f64::from(bounds_clone.width()),
          height: f64::from(bounds_clone.height()),
        },
      );

      w.setFrame_display(ns_rect, false);
    })
  }

  /// Repositions the layer within the window to the given rect and
  /// updates opacity.
  ///
  /// The `NSWindow` frame is never changed; only the `CALayer` moves.
  pub(crate) fn update(
    &self,
    rect: &Rect,
    opacity: Option<OpacityValue>,
  ) -> crate::Result<()> {
    let cg_origin_x = self.cg_origin_x;
    let cg_origin_y = self.cg_origin_y;
    let rect = rect.clone();

    self.layer.with(move |layer| {
      CATransaction::begin();
      CATransaction::setDisableActions(true);

      let frame = CGRect::new(
        CGPoint {
          x: f64::from(rect.x()) - cg_origin_x,
          y: f64::from(rect.y()) - cg_origin_y,
        },
        CGSize {
          width: f64::from(rect.width()),
          height: f64::from(rect.height()),
        },
      );

      layer.setFrame(frame);

      if let Some(opacity) = opacity {
        layer.setOpacity(opacity.0);
      }

      CATransaction::commit();
    })
  }

  /// Removes the overlay window from the screen.
  pub(crate) fn destroy(self) -> crate::Result<()> {
    self.ns_window.with(|w| w.orderOut(None))
  }
}
