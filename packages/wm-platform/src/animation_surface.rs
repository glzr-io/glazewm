use std::collections::HashMap;

#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2::runtime::AnyObject;
#[cfg(target_os = "macos")]
use objc2::{MainThreadMarker, MainThreadOnly};
#[cfg(target_os = "macos")]
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSScreen, NSWindow, NSWindowStyleMask,
};
#[cfg(target_os = "macos")]
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
#[cfg(target_os = "macos")]
use objc2_core_graphics::CGImage;
#[cfg(target_os = "macos")]
use objc2_foundation::NSRect;
#[cfg(target_os = "macos")]
use objc2_quartz_core::{CALayer, CATransaction};

use crate::OpacityValue;
#[cfg(target_os = "macos")]
use crate::{
  Dispatcher, NativeWindow, NativeWindowExtMacOs, Rect, ThreadBound,
};

/// Identifier for a layer within an `AnimationSurface`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LayerId(u64);

/// A transparent `NSWindow` with `CALayer` sublayers for animating window
/// screenshots.
///
/// This uses a single container window covering all screens, with one
/// `CALayer` per animation. Core Animation handles GPU compositing.
#[cfg(target_os = "macos")]
pub struct AnimationSurface {
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

#[cfg(target_os = "macos")]
impl AnimationSurface {
  /// Creates the container `NSWindow` spanning all screens.
  ///
  /// The window is transparent, ignores mouse events, and has its root
  /// layer's geometry flipped so sublayer origins match CG screen
  /// coordinates (top-left).
  #[allow(clippy::missing_panics_doc)]
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
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

  /// Screenshots the target window and adds a `CALayer` sublayer.
  ///
  /// Returns a `LayerId` handle for future updates and removal.
  pub fn add_layer(
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

  /// Updates frame and opacity for active layers in a single
  /// `CATransaction`.
  ///
  /// Implicit Core Animation animations are disabled so updates take
  /// effect immediately.
  pub fn update_layers(
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

  /// Removes a sublayer from the surface.
  pub fn remove_layer(&mut self, id: LayerId) -> crate::Result<()> {
    self.layers.with_mut(move |layers| {
      if let Some(layer) = layers.remove(&id) {
        layer.removeFromSuperlayer();
      }
    })
  }

  /// Returns whether the surface has any active layers.
  pub fn has_layers(&self) -> crate::Result<bool> {
    self.layers.with(|layers| !layers.is_empty())
  }

  /// Hides the container window without destroying it.
  ///
  /// The surface can be shown again later via `show`, avoiding the cost
  /// of recreating the `NSWindow` and root layer.
  pub fn hide(&self) -> crate::Result<()> {
    self.ns_window.with(|w| w.orderOut(None))
  }

  /// Shows the container window, bringing it to the front.
  ///
  /// Used to re-activate a previously hidden surface without
  /// recreating it.
  pub fn show(&self) -> crate::Result<()> {
    self.ns_window.with(|w| w.orderFrontRegardless())
  }

  /// Destroys the container window and all layers.
  pub fn destroy(self) -> crate::Result<()> {
    self.ns_window.with(|w| w.orderOut(None))
  }
}

#[cfg(target_os = "windows")]
use std::cell::{Cell, RefCell};
#[cfg(target_os = "windows")]
use std::sync::OnceLock;

#[cfg(target_os = "windows")]
use windows::{
  core::w,
  Win32::{
    Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM},
    Graphics::Gdi::{
      CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
      GetDC, ReleaseDC, SelectObject, SetStretchBltMode, StretchBlt,
      BLENDFUNCTION, HALFTONE, HDC, HGDIOBJ, SRCCOPY,
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW,
      ShowWindow, UpdateLayeredWindow, SW_HIDE, SW_SHOWNA, ULW_ALPHA,
      WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOPMOST,
      WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

#[cfg(target_os = "windows")]
use crate::{Dispatcher, NativeWindow, NativeWindowWindowsExt, Rect};

/// Guard ensuring the overlay window class is registered at most once per
/// process.
#[cfg(target_os = "windows")]
static OVERLAY_CLASS: OnceLock<()> = OnceLock::new();

#[cfg(target_os = "windows")]
struct OverlayState {
  /// Overlay `HWND` stored as `isize` (`HWND` is `!Send`).
  hwnd: isize,
  /// Off-screen memory DC holding the captured screenshot bitmap.
  hdc_mem: isize,
  /// The captured `HBITMAP` (selected into `hdc_mem`).
  hbitmap: isize,
  /// Width of the original screenshot, used for `StretchBlt` scaling.
  src_width: i32,
  /// Height of the original screenshot, used for `StretchBlt` scaling.
  src_height: i32,
  /// Current constant opacity (0–255). Uses `Cell` for interior
  /// mutability.
  alpha: Cell<u8>,
  /// Current overlay position and size. Uses `RefCell` for interior
  /// mutability.
  current_rect: RefCell<Rect>,
}

#[cfg(target_os = "windows")]
impl Drop for OverlayState {
  fn drop(&mut self) {
    unsafe {
      let _ = DestroyWindow(HWND(self.hwnd));
      let _ = DeleteDC(HDC(self.hdc_mem));
      let _ = DeleteObject(HGDIOBJ(self.hbitmap));
    }
  }
}

/// A borderless layered overlay window displaying a screenshot of a real
/// window.
#[cfg(target_os = "windows")]
pub struct OverlayWindow {
  inner: OverlayState,
}

#[cfg(target_os = "windows")]
unsafe impl Send for OverlayWindow {}
#[cfg(target_os = "windows")]
unsafe impl Sync for OverlayWindow {}

#[cfg(target_os = "windows")]
impl OverlayWindow {
  /// Creates a layered overlay `HWND` at `initial_rect` using
  /// pre-captured GDI handles `(hdc_mem, hbitmap)`.
  pub fn new(
    capture: (HDC, HGDIOBJ),
    initial_rect: &Rect,
  ) -> crate::Result<Self> {
    let rect = initial_rect.clone();

    // Register the overlay window class once per process.
    OVERLAY_CLASS.get_or_init(|| {
      let wnd_class = WNDCLASSW {
        lpszClassName: w!("GlazeWMOverlay"),
        lpfnWndProc: Some(Self::overlay_wnd_proc),
        ..Default::default()
      };
      unsafe { RegisterClassW(&raw const wnd_class) };
    });

    // Create the layered, always-on-top, non-activating overlay HWND.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_LAYERED
          | WS_EX_TOPMOST
          | WS_EX_NOACTIVATE
          | WS_EX_TRANSPARENT,
        w!("GlazeWMOverlay"),
        w!(""),
        WS_POPUP,
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        None,
        None,
        None,
        None,
      )
    };

    if hwnd.0 == 0 {
      return Err(crate::Error::Platform(
        "Failed to create overlay window.".to_string(),
      ));
    }

    let (hdc_mem, hbitmap) = capture;

    let inner = OverlayState {
      hwnd: hwnd.0,
      hdc_mem: hdc_mem.0,
      hbitmap: hbitmap.0,
      src_width: rect.width(),
      src_height: rect.height(),
      alpha: Cell::new(255),
      current_rect: RefCell::new(rect.clone()),
    };

    // Blit the screenshot and show the overlay (no-activate).
    update_layered(&inner, &rect);
    unsafe { ShowWindow(HWND(inner.hwnd), SW_SHOWNA) };

    Ok(Self { inner })
  }

  /// Moves and resizes the overlay to match `rect`. Dispatches to the
  /// event loop thread.
  pub fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    let rect = rect.clone();
    self.inner.current_rect.replace(rect.clone());
    update_layered(&self.inner, &rect);
    Ok(())
  }

  /// Sets overlay opacity (0.0–1.0). For fade animations.
  pub fn set_opacity(&self, alpha: f32) -> crate::Result<()> {
    let alpha = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    let rect = self.inner.current_rect.borrow().clone();
    update_layered(&self.inner, &rect);
    Ok(())
  }

  /// Hides the overlay without destroying Win32 resources.
  ///
  /// The overlay can be shown again via `show`.
  pub fn hide(&self) -> crate::Result<()> {
    unsafe { ShowWindow(HWND(self.inner.hwnd), SW_HIDE) }?;
    Ok(())
  }

  /// Shows a previously hidden overlay without activating it.
  pub fn show(&self) -> crate::Result<()> {
    unsafe { ShowWindow(HWND(self.inner.hwnd), SW_SHOWNA) }?;
    Ok(())
  }

  pub fn destroy(self) -> crate::Result<()> {
    // TODO: Destroy the window.
    unsafe { ShowWindow(HWND(self.inner.hwnd), SW_HIDE) }?;
    Ok(())
  }

  /// Window procedure for the overlay class.
  #[cfg(target_os = "windows")]
  unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
  }
}

/// Blits the stored screenshot (scaled to `rect`) into the layered overlay
/// window via `UpdateLayeredWindow`.
#[cfg(target_os = "windows")]
fn update_layered(state: &OverlayState, rect: &Rect) {
  let new_w = rect.width();
  let new_h = rect.height();
  let x = rect.x();
  let y = rect.y();

  // Guard against zero-size rects which would cause GDI errors.
  if new_w <= 0 || new_h <= 0 {
    return;
  }

  unsafe {
    let hwnd = HWND(state.hwnd);
    let screen_dc = GetDC(HWND(0));

    // Create a temporary scaled DC for this frame.
    let hdc_scaled = CreateCompatibleDC(screen_dc);
    let hbmp_scaled = CreateCompatibleBitmap(screen_dc, new_w, new_h);
    let hbmp_old = SelectObject(hdc_scaled, HGDIOBJ(hbmp_scaled.0));

    // Scale the captured screenshot to the target size.
    SetStretchBltMode(hdc_scaled, HALFTONE);
    let _ = StretchBlt(
      hdc_scaled,
      0,
      0,
      new_w,
      new_h,
      HDC(state.hdc_mem),
      0,
      0,
      state.src_width,
      state.src_height,
      SRCCOPY,
    );

    // AC_SRC_OVER = 0 (blend mode: source over).
    let blend = BLENDFUNCTION {
      BlendOp: 0,
      BlendFlags: 0,
      SourceConstantAlpha: state.alpha.get(),
      AlphaFormat: 0,
    };
    let pt_dst = POINT { x, y };
    let sz = SIZE {
      cx: new_w,
      cy: new_h,
    };
    let pt_src = POINT { x: 0, y: 0 };

    let _ = UpdateLayeredWindow(
      hwnd,
      HDC(0),
      Some(&raw const pt_dst),
      Some(&raw const sz),
      hdc_scaled,
      Some(&raw const pt_src),
      COLORREF(0),
      Some(&raw const blend),
      ULW_ALPHA,
    );

    // Restore the old bitmap and clean up the temporary scaled DC.
    SelectObject(hdc_scaled, hbmp_old);
    let _ = DeleteObject(HGDIOBJ(hbmp_scaled.0));
    let _ = DeleteDC(hdc_scaled);
    ReleaseDC(HWND(0), screen_dc);
  }
}

#[cfg(target_os = "windows")]
impl std::fmt::Debug for OverlayWindow {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OverlayWindow").finish_non_exhaustive()
  }
}

/// A collection of layered overlay windows for animating window
/// screenshots.
#[cfg(target_os = "windows")]
pub struct AnimationSurface {
  layers: HashMap<LayerId, OverlayWindow>,
  next_id: u64,
  dispatcher: Dispatcher,
}

#[cfg(target_os = "windows")]
impl AnimationSurface {
  /// Creates a new, empty `AnimationSurface`.
  ///
  /// No `HWND` is allocated until the first `add_layer` call.
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    Ok(Self {
      layers: HashMap::new(),
      next_id: 0,
      dispatcher: dispatcher.clone(),
    })
  }

  /// Screenshots the target window and adds an `OverlayWindow` layer.
  ///
  /// Returns a `LayerId` handle for future updates and removal.
  pub fn add_layer(
    &mut self,
    window: &NativeWindow,
    rect: &Rect,
    opacity: Option<f32>,
  ) -> crate::Result<LayerId> {
    let capture = window.screen_capture(rect)?;
    let overlay = OverlayWindow::new(capture, rect, &self.dispatcher)?;

    if let Some(alpha) = opacity {
      if let Err(err) = overlay.set_opacity(alpha) {
        tracing::warn!("Failed to set initial overlay opacity: {}", err);
      }
    }

    let id = LayerId(self.next_id);
    self.next_id += 1;
    self.layers.insert(id, overlay);

    Ok(id)
  }

  /// Updates frame and opacity for a set of active layers.
  pub fn update_layers(
    &self,
    updates: Vec<(LayerId, Rect, Option<OpacityValue>)>,
  ) -> crate::Result<()> {
    for (id, rect, opacity) in &updates {
      if let Some(overlay) = self.layers.get(id) {
        if let Err(err) = overlay.set_frame(rect) {
          tracing::warn!("Failed to update overlay frame: {}", err);
        }

        if let Some(opacity) = opacity {
          if let Err(err) = overlay.set_opacity(opacity.to_f32()) {
            tracing::warn!("Failed to update overlay opacity: {}", err);
          }
        }
      }
    }

    Ok(())
  }

  /// Removes a layer from the surface and destroys its `HWND`.
  pub fn remove_layer(&mut self, id: LayerId) -> crate::Result<()> {
    if let Some(overlay) = self.layers.remove(&id) {
      if let Err(err) = overlay.destroy() {
        tracing::warn!("Failed to destroy overlay layer: {}", err);
      }
    }

    Ok(())
  }

  /// Returns whether the surface has any active layers.
  pub fn has_layers(&self) -> crate::Result<bool> {
    Ok(!self.layers.is_empty())
  }

  /// No-op on Windows — each layer's `HWND` is shown on creation.
  pub fn show(&self) -> crate::Result<()> {
    Ok(())
  }

  /// No-op on Windows — layers are destroyed individually via
  /// `remove_layer`.
  pub fn hide(&self) -> crate::Result<()> {
    Ok(())
  }

  /// Destroys all layers and their associated `HWND`s.
  pub fn destroy(mut self) -> crate::Result<()> {
    for (_, overlay) in self.layers.drain() {
      if let Err(err) = overlay.destroy() {
        tracing::warn!(
          "Failed to destroy overlay during surface teardown: {}",
          err
        );
      }
    }

    Ok(())
  }
}

impl std::fmt::Debug for AnimationSurface {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AnimationSurface").finish_non_exhaustive()
  }
}
