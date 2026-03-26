use std::collections::HashMap;

use objc2::{
  rc::Retained, runtime::AnyObject, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSScreen, NSWindow, NSWindowStyleMask,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGImage;
use objc2_foundation::NSRect;
use objc2_quartz_core::{CALayer, CATransaction};

use crate::{
  Dispatcher, LayerId, NativeWindow, NativeWindowExtMacOs, OpacityValue,
  Rect, ThreadBound,
};

/// Platform-specific implementation of [`AnimationSurface`].
pub(crate) struct AnimationSurface {
  ns_window: ThreadBound<Retained<NSWindow>>,
  root_layer: ThreadBound<Retained<CALayer>>,
  layers: ThreadBound<HashMap<LayerId, Retained<CALayer>>>,
  next_id: u64,
  /// Top-left of the container window in CG (screen) coordinates.
  cg_origin_x: f64,
  cg_origin_y: f64,
  /// Backing scale factor for Retina content.
  scale_factor: f64,
}

impl AnimationSurface {
  /// Creates the container `NSWindow` spanning all screens.
  ///
  /// The window is transparent, ignores mouse events, and has its root
  /// layer's geometry flipped so sublayer origins match CG screen
  /// coordinates (top-left).
  #[allow(clippy::missing_panics_doc)]
  pub(crate) fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let dispatcher_clone = dispatcher.clone();

    let (
      ns_window,
      root_layer,
      layers,
      cg_origin_x,
      cg_origin_y,
      scale_factor,
    ) = dispatcher.dispatch_sync(move || {
      // SAFETY: `dispatch_sync` runs on the main thread.
      let mtm = unsafe { MainThreadMarker::new_unchecked() };

      let screens = NSScreen::screens(mtm);
      let primary_height =
        screens.iter().next().map_or(0.0, |s| s.frame().size.height);

      let scale_factor = screens
        .iter()
        .next()
        .map_or(2.0, |s| s.backingScaleFactor());

      // Compute union of all screen frames in AppKit coordinates.
      let (mut min_x, mut min_y) = (f64::MAX, f64::MAX);
      let (mut max_x, mut max_y) = (f64::MIN, f64::MIN);
      for screen in &screens {
        let f = screen.frame();
        min_x = min_x.min(f.origin.x);
        min_y = min_y.min(f.origin.y);
        max_x = max_x.max(f.origin.x + f.size.width);
        max_y = max_y.max(f.origin.y + f.size.height);
      }

      let ns_rect = NSRect::new(
        objc2_foundation::NSPoint { x: min_x, y: min_y },
        objc2_foundation::NSSize {
          width: max_x - min_x,
          height: max_y - min_y,
        },
      );

      // Window's top-left in CG (screen) coordinates.
      // TODO: Origin Y includes the height of the title bar.
      let cg_origin_x = min_x;
      let cg_origin_y = primary_height - max_y;

      // Create borderless transparent NSWindow.
      let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
          NSWindow::alloc(mtm),
          ns_rect,
          NSWindowStyleMask::Borderless,
          NSBackingStoreType::Buffered,
          false,
        )
      };

      window.setBackgroundColor(Some(&NSColor::clearColor()));
      window.setOpaque(false);
      window.setIgnoresMouseEvents(true);
      // Place above all normal windows (NSStatusWindowLevel = 25).
      window.setLevel(25);
      // SAFETY: We manage lifetime via `ThreadBound` + `orderOut`.
      unsafe { window.setReleasedWhenClosed(false) };

      // Enable Core Animation layer backing.
      if let Some(content_view) = window.contentView() {
        content_view.setWantsLayer(true);
      }

      let root_layer = window
        .contentView()
        .and_then(|v| v.layer())
        .expect("layer must exist after setWantsLayer");

      // Flip the y-axis, so that layers can use CG screen coordinates.
      root_layer.setGeometryFlipped(true);

      window.orderFrontRegardless();

      (
        ThreadBound::new(window, dispatcher_clone.clone()),
        ThreadBound::new(root_layer, dispatcher_clone.clone()),
        ThreadBound::new(HashMap::new(), dispatcher_clone),
        cg_origin_x,
        cg_origin_y,
        scale_factor,
      )
    })?;

    Ok(Self {
      ns_window,
      root_layer,
      layers,
      next_id: 0,
      cg_origin_x,
      cg_origin_y,
      scale_factor,
    })
  }

  /// Implements [`AnimationSurface::add_layer`].
  pub(crate) fn add_layer(
    &mut self,
    window: &NativeWindow,
    rect: &Rect,
    opacity: Option<f32>,
  ) -> crate::Result<LayerId> {
    let rect = rect.clone();
    let cg_image = window.screen_capture()?;

    let scale_factor = self.scale_factor;
    let cg_origin_x = self.cg_origin_x;
    let cg_origin_y = self.cg_origin_y;
    let root_layer = &self.root_layer;

    let id = LayerId(self.next_id);
    self.next_id += 1;

    self
      .layers
      .with_mut(move |layers| -> crate::Result<LayerId> {
        let layer = CALayer::new();

        // Set screenshot as layer contents.
        // SAFETY: `CGImageRef` is accepted by `CALayer.contents`.
        unsafe {
          layer.setContents(Some(
            &*std::ptr::from_ref::<CGImage>(&cg_image).cast::<AnyObject>(),
          ));
        };

        // Get frame using the size of the captured image.
        let frame = {
          #[allow(clippy::cast_precision_loss)]
          let image_width =
            CGImage::width(Some(&cg_image)) as f64 / scale_factor;

          #[allow(clippy::cast_precision_loss)]
          let image_height =
            CGImage::height(Some(&cg_image)) as f64 / scale_factor;

          CGRect::new(
            CGPoint {
              x: f64::from(rect.x()) - cg_origin_x,
              // Offset by the height of the title bar.
              y: f64::from(rect.y()) - cg_origin_y + 25.,
            },
            CGSize {
              width: image_width,
              height: image_height,
            },
          )
        };

        layer.setFrame(frame);

        if let Some(alpha) = opacity {
          layer.setOpacity(alpha);
        }

        let root_layer = root_layer.get_ref()?;
        root_layer.addSublayer(&layer);
        layers.insert(id, layer);

        Ok(id)
      })?
  }

  /// Implements [`AnimationSurface::update_layers`].
  pub(crate) fn update_layers(
    &self,
    updates: Vec<(LayerId, Rect, Option<OpacityValue>)>,
  ) -> crate::Result<()> {
    let cg_origin_x = self.cg_origin_x;
    let cg_origin_y = self.cg_origin_y;

    self.layers.with(move |layers| -> crate::Result<()> {
      CATransaction::begin();
      CATransaction::setDisableActions(true);

      for (id, rect, opacity) in &updates {
        if let Some(layer) = layers.get(id) {
          let frame = CGRect::new(
            CGPoint {
              x: f64::from(rect.x()) - cg_origin_x,
              // Offset by the height of the title bar.
              y: f64::from(rect.y()) - cg_origin_y + 25.,
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
        }
      }

      CATransaction::commit();
      Ok(())
    })?
  }

  /// Implements [`AnimationSurface::remove_layer`].
  pub(crate) fn remove_layer(&mut self, id: LayerId) -> crate::Result<()> {
    self.layers.with_mut(move |layers| {
      if let Some(layer) = layers.remove(&id) {
        layer.removeFromSuperlayer();
      }
    })
  }

  /// Implements [`AnimationSurface::has_layers`].
  pub(crate) fn has_layers(&self) -> crate::Result<bool> {
    self.layers.with(|layers| !layers.is_empty())
  }

  /// Implements [`AnimationSurface::hide`].
  pub(crate) fn hide(&self) -> crate::Result<()> {
    self.ns_window.with(|w| w.orderOut(None))
  }

  /// Implements [`AnimationSurface::show`].
  pub(crate) fn show(&self) -> crate::Result<()> {
    self.ns_window.with(|w| w.orderFrontRegardless())
  }

  /// Implements [`AnimationSurface::destroy`].
  pub(crate) fn destroy(self) -> crate::Result<()> {
    self.ns_window.with(|w| w.orderOut(None))
  }
}
