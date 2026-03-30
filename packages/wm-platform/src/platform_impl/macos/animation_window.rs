use objc2::{
  rc::Retained, runtime::AnyObject, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSScreen, NSWindow, NSWindowOrderingMode,
  NSWindowStyleMask,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGImage;
use objc2_foundation::NSRect;
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
  /// Top-left of the animation window in CG (screen) coordinates.
  cg_origin_x: f64,
  cg_origin_y: f64,
  /// Height of the overlay window in points.
  bounds_height: f64,
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
    let source_window_number = window.id().0;
    let inner_rect = inner_rect.clone();
    let opacity = opacity.map(|o| o.0);

    let dispatcher_clone = dispatcher.clone();

    let (ns_window, layer, cg_origin_x, cg_origin_y, bounds_height) =
      dispatcher.dispatch_sync(move || {
        // SAFETY: `dispatch_sync` runs on the main thread.
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        // TODO: Clean up the coordinates logic.
        let screens = NSScreen::screens(mtm);
        let primary_height = screens
          .iter()
          .next()
          .map_or(0.0, |screen| screen.frame().size.height);

        let scale_factor = screens
          .iter()
          .next()
          .map_or(2.0, |screen| screen.backingScaleFactor());

        let cg_origin_x = f64::from(outer_rect.x());
        let cg_origin_y = f64::from(outer_rect.y());
        let bounds_height = f64::from(outer_rect.height());

        // Convert CG coordinates (top-left origin) to AppKit
        // (bottom-left).
        let appkit_y = primary_height
          - f64::from(outer_rect.y())
          - f64::from(outer_rect.height());

        let ns_rect = NSRect::new(
          objc2_foundation::NSPoint {
            x: f64::from(outer_rect.x()),
            y: appkit_y,
          },
          objc2_foundation::NSSize {
            width: f64::from(outer_rect.width()),
            height: f64::from(outer_rect.height()),
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

        let layer = CALayer::new();

        // SAFETY: `CGImageRef` is accepted by `CALayer.contents`.
        unsafe {
          layer.setContents(Some(
            &*std::ptr::from_ref::<CGImage>(&cg_image).cast::<AnyObject>(),
          ));
        };

        #[allow(clippy::cast_precision_loss)]
        let image_width =
          CGImage::width(Some(&cg_image)) as f64 / scale_factor;
        #[allow(clippy::cast_precision_loss)]
        let image_height =
          CGImage::height(Some(&cg_image)) as f64 / scale_factor;

        // Convert CG y (top-left origin, y-down) to AppKit layer
        // coordinates (bottom-left origin, y-up).
        let frame = CGRect::new(
          CGPoint {
            x: f64::from(inner_rect.x()) - cg_origin_x,
            y: bounds_height
              - (f64::from(inner_rect.y()) - cg_origin_y)
              - image_height,
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
          bounds_height,
        )
      })?;

    Ok(Self {
      ns_window,
      layer,
      cg_origin_x,
      cg_origin_y,
      bounds_height,
    })
  }

  /// Resizes the `NSWindow` to cover the union of `start_rect` and
  /// `target_rect`, updating the stored CG origin.
  ///
  /// Called when an animation's target changes mid-flight so the
  /// existing screenshot and z-order are preserved.
  pub(crate) fn resize(&mut self, outer_rect: &Rect) -> crate::Result<()> {
    self.cg_origin_x = f64::from(outer_rect.x());
    self.cg_origin_y = f64::from(outer_rect.y());
    self.bounds_height = f64::from(outer_rect.height());

    let outer_rect = outer_rect.clone();

    self.ns_window.with(move |ns_window| {
      // SAFETY: `with` runs on the main thread.
      let mtm = unsafe { MainThreadMarker::new_unchecked() };

      let primary_height = NSScreen::screens(mtm)
        .iter()
        .next()
        .map_or(0.0, |s| s.frame().size.height);

      let appkit_y = primary_height
        - f64::from(outer_rect.y())
        - f64::from(outer_rect.height());

      let ns_rect = NSRect::new(
        objc2_foundation::NSPoint {
          x: f64::from(outer_rect.x()),
          y: appkit_y,
        },
        objc2_foundation::NSSize {
          width: f64::from(outer_rect.width()),
          height: f64::from(outer_rect.height()),
        },
      );

      ns_window.setFrame_display(ns_rect, false);
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
    // Convert CG y (top-left origin, y-down) to AppKit layer
    // coordinates (bottom-left origin, y-up).
    let frame = CGRect::new(
      CGPoint {
        x: f64::from(inner_rect.x()) - self.cg_origin_x,
        y: self.bounds_height
          - (f64::from(inner_rect.y()) - self.cg_origin_y)
          - f64::from(inner_rect.height()),
      },
      CGSize {
        width: f64::from(inner_rect.width()),
        height: f64::from(inner_rect.height()),
      },
    );

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
