use objc2::{
  rc::Retained, runtime::AnyObject, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSWindow, NSWindowOrderingMode,
  NSWindowStyleMask,
};
use objc2_core_foundation::{CFRetained, CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGImage;
#[allow(deprecated)]
use objc2_core_graphics::{
  CGWindowImageOption, CGWindowListCreateImage, CGWindowListOption,
};
use objc2_quartz_core::{CALayer, CATransaction};

use crate::{
  Dispatcher, NativeWindow, OpacityValue, Rect, ThreadBound, WindowId,
};

/// Platform-specific implementation of [`AnimationContext`].
pub(crate) struct AnimationContext;

impl AnimationContext {
  /// Implements [`AnimationContext::new`].
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn new(_dispatcher: &Dispatcher) -> crate::Result<Self> {
    Ok(Self)
  }

  /// Implements [`AnimationContext::transaction`].
  #[allow(clippy::unused_self)]
  pub(crate) fn transaction<F, R>(
    &self,
    update_fn: F,
    dispatcher: &Dispatcher,
  ) -> crate::Result<R>
  where
    F: FnOnce() -> R + Send,
    R: Send,
  {
    dispatcher.dispatch_sync(|| {
      CATransaction::begin();
      CATransaction::setDisableActions(true);
      let result = update_fn();
      CATransaction::commit();
      result
    })
  }
}

/// Platform-specific implementation of [`AnimationWindow`].
pub(crate) struct AnimationWindow {
  ns_window: ThreadBound<Retained<NSWindow>>,
  layer: ThreadBound<Retained<CALayer>>,

  /// Frame of the `AnimationWindow` (in CG coordinates).
  outer_rect: Rect,

  /// Height of the primary display.
  display_height: i32,
}

impl AnimationWindow {
  /// Implements [`AnimationWindow::new`].
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

    let captured = CapturedFrame::new(window.id())?;

    let (ns_window, layer, display_height) =
      dispatcher.dispatch_sync(|| -> DispatchResult {
        // SAFETY: `Dispatcher::dispatch_sync` runs on the main thread.
        let mtm = unsafe { MainThreadMarker::new_unchecked() };

        // Get height of the primary display, needed for CG<->AppKit
        // coordinate conversion.
        let display_height =
          dispatcher.primary_display()?.bounds()?.height();

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
        // `Retained<NSWindow>` field is dropped, it will also send a
        // release call and segfault.
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
            &*std::ptr::from_ref::<CGImage>(&captured.cg_image)
              .cast::<AnyObject>(),
          ));
        };

        let scale_factor =
          dispatcher.nearest_display(window)?.scale_factor()?;

        // Image needs to be scaled by the scale factor of the display
        // containing the window.
        #[allow(clippy::cast_precision_loss)]
        let image_width =
          CGImage::width(Some(&captured.cg_image)) as f32 / scale_factor;
        #[allow(clippy::cast_precision_loss)]
        let image_height =
          CGImage::height(Some(&captured.cg_image)) as f32 / scale_factor;

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

  /// Implements [`AnimationWindow::resize`].
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

  /// Implements [`AnimationWindow::destroy`].
  pub(crate) fn destroy(self) -> crate::Result<()> {
    self.ns_window.with(|ns_window| ns_window.close())
  }

  /// Updates the `CALayer` position and opacity within the window.
  ///
  /// The window's frame isn't changed; only the layer with the screen
  /// screen capture is updated.
  ///
  /// Shared by [`AnimationWindow::new`] and [`AnimationWindow::update`].
  /// Must be called inside `AnimationContext::transaction`.
  fn update_layer(
    layer: &Retained<CALayer>,
    inner_rect: &Rect,
    outer_rect: &Rect,
    opacity: Option<&OpacityValue>,
  ) {
    // `inner_rect` needs to be positioned relative to the window's frame.
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

/// A screen capture of a window via `CGWindowListCreateImage`.
struct CapturedFrame {
  cg_image: CFRetained<CGImage>,
}

impl CapturedFrame {
  /// Captures a single frame of a given window.
  #[allow(deprecated)]
  fn new(window_id: WindowId) -> crate::Result<Self> {
    // Use `CGRectNull` to capture the minimum rectangle that encloses the
    // window. See: https://developer.apple.com/documentation/coregraphics/cgwindowlistcreateimage(_:_:_:_:)
    let cg_rect_null = CGRect::new(
      CGPoint {
        x: f64::INFINITY,
        y: f64::INFINITY,
      },
      CGSize::ZERO,
    );

    // NOTE: `CGWindowListCreateImage` is deprecated, but functional.
    // ScreenCaptureKit is recommended instead, see:
    // https://developer.apple.com/documentation/screencapturekit/scwindow
    let image = CGWindowListCreateImage(
      cg_rect_null,
      CGWindowListOption::OptionIncludingWindow,
      window_id.0,
      CGWindowImageOption::BestResolution
        .union(CGWindowImageOption::BoundsIgnoreFraming),
    )
    .ok_or(crate::Error::Platform(
      "Failed to create window screenshot.".to_string(),
    ))?;

    Ok(Self { cg_image: image })
  }
}
