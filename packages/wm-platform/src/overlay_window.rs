#[cfg(target_os = "macos")]
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
#[allow(deprecated)]
use objc2_core_graphics::{
  CGWindowImageOption, CGWindowListCreateImage, CGWindowListOption,
};
#[cfg(target_os = "macos")]
use objc2_foundation::NSRect;
#[cfg(target_os = "macos")]
use objc2_quartz_core::{kCAGravityTopLeft, CALayer, CATransaction};

use crate::OpacityValue;
#[cfg(target_os = "macos")]
use crate::{Dispatcher, Rect, ThreadBound, WindowId};

// ── macOS: AnimationSurface
// ────────────────────────────────────────────────

/// Opaque handle identifying a layer within an `AnimationSurface`.
#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LayerId(u64);

/// A group of layers for a single animation entry.
///
/// Contains a blurred background that stretches to fill the frame and a
/// sharp foreground that preserves the original screenshot dimensions.
#[cfg(target_os = "macos")]
struct AnimationLayerGroup {
  /// Parent layer added to the root.
  group: Retained<CALayer>,
  /// Dark transparent overlay sublayer.
  dim: Retained<CALayer>,
  /// Sublayer with `kCAGravityTopLeft`, clipped to bounds.
  sharp: Retained<CALayer>,
}

/// Per-surface state, bound to the event loop thread.
#[cfg(target_os = "macos")]
struct AnimationSurfaceInner {
  ns_window: Retained<NSWindow>,
  root_layer: Retained<CALayer>,
  layers: HashMap<LayerId, AnimationLayerGroup>,
  next_id: u64,
  /// Top-left of the container window in CG (screen) coordinates.
  cg_origin_x: f64,
  cg_origin_y: f64,
  /// Backing scale factor for Retina content.
  scale_factor: f64,
}

/// A single transparent `NSWindow` with `CALayer` sublayers for animating
/// window screenshots.
///
/// Instead of one `NSWindow` per animating window, this uses a single
/// container window covering all screens, with one `CALayer` per
/// animation. Core Animation handles GPU compositing.
#[cfg(target_os = "macos")]
pub struct AnimationSurface {
  inner: ThreadBound<AnimationSurfaceInner>,
}

#[cfg(target_os = "macos")]
impl AnimationSurface {
  /// Creates the container `NSWindow` spanning all screens.
  ///
  /// The window is transparent, ignores mouse events, and has its root
  /// layer's geometry flipped so sublayer origins match CG screen
  /// coordinates (top-left).
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let disp = dispatcher.clone();

    let inner = dispatcher.dispatch_sync(move || {
      // SAFETY: `dispatch_sync` runs on the event loop (main) thread.
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

      // Flip geometry so sublayer y=0 is at the top, matching CG
      // screen coordinates.
      root_layer.setGeometryFlipped(true);

      window.orderFrontRegardless();

      ThreadBound::new(
        AnimationSurfaceInner {
          ns_window: window,
          root_layer,
          layers: HashMap::new(),
          next_id: 0,
          cg_origin_x,
          cg_origin_y,
          scale_factor,
        },
        disp,
      )
    })?;

    Ok(Self { inner })
  }

  /// Screenshots the target window and adds a `CALayer` sublayer.
  ///
  /// Returns a `LayerId` handle for future updates and removal.
  #[allow(deprecated)]
  pub fn add_layer(
    &mut self,
    window_id: WindowId,
    rect: &Rect,
    opacity: Option<f32>,
  ) -> crate::Result<LayerId> {
    let wid = window_id.0;
    let rect = rect.clone();

    self.inner.with_mut(move |inner| {
      // Screenshot the target window.
      let cg_rect = CGRect::new(
        CGPoint {
          x: f64::from(rect.x()),
          y: f64::from(rect.y()),
        },
        CGSize {
          width: f64::from(rect.width()),
          height: f64::from(rect.height()),
        },
      );

      // NOTE: `CGWindowListCreateImage` is deprecated, but functional.
      // ScreenCaptureKit is recommended instead, see: https://developer.apple.com/documentation/screencapturekit/scwindow.
      let cg_image = CGWindowListCreateImage(
        cg_rect,
        CGWindowListOption::OptionIncludingWindow,
        wid,
        CGWindowImageOption::BestResolution,
      );

      // Shared contents pointer for both sublayers.
      let contents_ptr: Option<*const AnyObject> =
        cg_image.as_ref().map(|img| {
          // SAFETY: `CGImageRef` is accepted by `CALayer.contents` as
          // a toll-free-bridged Core Foundation type.
          let img_ref: &objc2_core_graphics::CGImage = img;
          let ptr: *const AnyObject =
            (img_ref as *const objc2_core_graphics::CGImage).cast();
          ptr
        });

      let bounds = CGRect::new(
        CGPoint { x: 0.0, y: 0.0 },
        CGSize {
          width: f64::from(rect.width()),
          height: f64::from(rect.height()),
        },
      );

      // Position in root layer coordinates (geometry-flipped, top-left
      // origin).
      let frame = CGRect::new(
        CGPoint {
          x: f64::from(rect.x()) - inner.cg_origin_x,
          y: f64::from(rect.y()) - inner.cg_origin_y,
        },
        CGSize {
          width: f64::from(rect.width()),
          height: f64::from(rect.height()),
        },
      );

      // --- Group layer (container) ---
      let group = CALayer::new();
      group.setFrame(frame);
      group.setMasksToBounds(true);

      // --- Dim sublayer (dark transparent overlay) ---
      let dim = CALayer::new();
      dim.setFrame(bounds);
      dim.setBackgroundColor(Some(
        &objc2_core_graphics::CGColor::new_generic_gray(0.0, 0.4),
      ));

      group.addSublayer(&dim);

      // --- Sharp sublayer (original size, clipped) ---
      let sharp = CALayer::new();
      if let Some(ptr) = contents_ptr {
        // SAFETY: pointer derived from a valid `CGImage` above.
        unsafe { sharp.setContents(Some(&*ptr)) };
      }
      sharp.setContentsScale(inner.scale_factor);
      // SAFETY: `kCAGravityTopLeft` is a valid Core Animation constant.
      sharp.setContentsGravity(unsafe { kCAGravityTopLeft });
      sharp.setFrame(bounds);
      sharp.setMasksToBounds(true);

      group.addSublayer(&sharp);

      if let Some(alpha) = opacity {
        group.setOpacity(alpha);
      }

      inner.root_layer.addSublayer(&group);

      let id = LayerId(inner.next_id);
      inner.next_id += 1;
      inner.layers.insert(
        id,
        AnimationLayerGroup { group, dim, sharp },
      );

      id
    })
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
    self.inner.with(move |inner| {
      CATransaction::begin();
      CATransaction::setDisableActions(true);

      for (id, rect, opacity) in &updates {
        if let Some(entry) = inner.layers.get(id) {
          let frame = CGRect::new(
            CGPoint {
              x: f64::from(rect.x()) - inner.cg_origin_x,
              y: f64::from(rect.y()) - inner.cg_origin_y,
            },
            CGSize {
              width: f64::from(rect.width()),
              height: f64::from(rect.height()),
            },
          );
          entry.group.setFrame(frame);

          // Sublayers fill the group's bounds.
          let bounds = CGRect::new(
            CGPoint { x: 0.0, y: 0.0 },
            CGSize {
              width: f64::from(rect.width()),
              height: f64::from(rect.height()),
            },
          );
          entry.dim.setFrame(bounds);
          entry.sharp.setFrame(bounds);

          if let Some(opacity) = opacity {
            entry.group.setOpacity(opacity.to_f32());
          }
        }
      }

      CATransaction::commit();
    })
  }

  /// Removes a sublayer from the surface.
  pub fn remove_layer(&mut self, id: LayerId) -> crate::Result<()> {
    self.inner.with_mut(move |inner| {
      if let Some(entry) = inner.layers.remove(&id) {
        entry.group.removeFromSuperlayer();
      }
    })
  }

  /// Returns whether the surface has any active layers.
  pub fn has_layers(&self) -> crate::Result<bool> {
    self.inner.with(|inner| !inner.layers.is_empty())
  }

  /// Destroys the container window and all layers.
  pub fn destroy(self) -> crate::Result<()> {
    self.inner.with(|inner| {
      inner.ns_window.orderOut(None);
    })
  }
}

#[cfg(target_os = "macos")]
impl std::fmt::Debug for AnimationSurface {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AnimationSurface").finish_non_exhaustive()
  }
}

// ── Windows: OverlayWindow + move_group
// ────────────────────────────────────────────────

/// Batches frame and opacity updates for multiple overlays.
///
/// On Windows there is no transaction API, so updates are applied
/// individually.
#[cfg(target_os = "windows")]
pub fn move_group(
  overlays: &[(&OverlayWindow, &Rect, Option<OpacityValue>)],
) {
  for (overlay, rect, opacity) in overlays {
    let _ = overlay.set_frame(rect);

    if let Some(opacity) = opacity {
      let _ = overlay.set_opacity(opacity.to_f32());
    }
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
use crate::{Dispatcher, Rect, ThreadBound, WindowId};

// Raw declaration for `PrintWindow`, which is absent from windows-rs 0.52
// bindings. Provided by `user32.dll` (already linked via the windows
// crate).
#[cfg(target_os = "windows")]
extern "system" {
  fn PrintWindow(hwnd: HWND, hdcblt: HDC, nflags: u32) -> i32;
}

/// `PW_RENDERFULLCONTENT` — instructs `PrintWindow` to capture
/// DWM-composited content such as DirectX surfaces.
#[cfg(target_os = "windows")]
const PW_RENDERFULLCONTENT: u32 = 2;

/// Default window procedure wrapper for the overlay class.
///
/// Required because `DefWindowProcW` in windows-rs 0.52 is a generic
/// function and cannot be used as a bare function pointer.
#[cfg(target_os = "windows")]
unsafe extern "system" fn overlay_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  DefWindowProcW(hwnd, msg, wparam, lparam)
}

/// Guard ensuring the overlay window class is registered at most once per
/// process.
#[cfg(target_os = "windows")]
static OVERLAY_CLASS: OnceLock<()> = OnceLock::new();

/// Per-overlay Win32 resources, bound to the event loop thread via
/// `ThreadBound`.
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
  /// Destroys Win32 resources. Runs on the event loop thread, guaranteed
  /// by `ThreadBound::drop`.
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
///
/// Used for smooth animations — animating our own layered window with
/// `UpdateLayeredWindow` is cheaper than re-positioning third-party
/// windows on every frame.
#[cfg(target_os = "windows")]
pub struct OverlayWindow {
  inner: ThreadBound<OverlayState>,
}

#[cfg(target_os = "windows")]
impl OverlayWindow {
  /// Screenshots the source window and creates a layered overlay `HWND`
  /// at `initial_rect`.
  ///
  /// If `PrintWindow` fails the overlay is still created; the window will
  /// appear fully transparent until destroyed.
  pub fn new(
    window_id: WindowId,
    initial_rect: &Rect,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let src_hwnd = window_id.0;
    let rect = initial_rect.clone();
    let disp = dispatcher.clone();

    let inner = dispatcher.dispatch_sync(move || {
      // Register the overlay window class once per process.
      OVERLAY_CLASS.get_or_init(|| {
        let wnd_class = WNDCLASSW {
          lpszClassName: w!("GlazeWMOverlay"),
          lpfnWndProc: Some(overlay_wnd_proc),
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

      // Capture the source window content into an off-screen DC + bitmap.
      let (hdc_mem, hbitmap) = unsafe {
        let screen_dc = GetDC(HWND(0));
        let hdc = CreateCompatibleDC(screen_dc);
        let bmp =
          CreateCompatibleBitmap(screen_dc, rect.width(), rect.height());

        // Select the bitmap into the DC before rendering into it.
        SelectObject(hdc, HGDIOBJ(bmp.0));

        // Capture DWM-composited window content. Non-fatal on failure.
        // SAFETY: `PrintWindow` is a stable Win32 API from user32.dll.
        let _ = PrintWindow(HWND(src_hwnd), hdc, PW_RENDERFULLCONTENT);

        ReleaseDC(HWND(0), screen_dc);
        (hdc, bmp)
      };

      let state = OverlayState {
        hwnd: hwnd.0,
        hdc_mem: hdc_mem.0,
        hbitmap: hbitmap.0,
        src_width: rect.width(),
        src_height: rect.height(),
        alpha: Cell::new(255),
        current_rect: RefCell::new(rect.clone()),
      };

      // Blit the screenshot and show the overlay (no-activate).
      update_layered(&state, &rect);
      unsafe { ShowWindow(HWND(state.hwnd), SW_SHOWNA) };

      Ok(ThreadBound::new(state, disp))
    })??;

    Ok(Self { inner })
  }

  /// Moves and resizes the overlay to match `rect`. Dispatches to the
  /// event loop thread.
  pub fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    let rect = rect.clone();
    self.inner.with(move |state| {
      state.current_rect.replace(rect.clone());
      update_layered(state, &rect);
    })
  }

  /// Sets overlay opacity (0.0–1.0). For fade animations.
  pub fn set_opacity(&self, alpha: f32) -> crate::Result<()> {
    self.inner.with(move |state| {
      state
        .alpha
        .set((alpha.clamp(0.0, 1.0) * 255.0).round() as u8);
      let rect = state.current_rect.borrow().clone();
      update_layered(state, &rect);
    })
  }

  /// Hides the overlay and schedules its Win32 resources for destruction
  /// on the event loop thread.
  pub fn destroy(self) -> crate::Result<()> {
    self.inner.with(|state| unsafe {
      let _ = ShowWindow(HWND(state.hwnd), SW_HIDE);
    })?;
    // `ThreadBound::drop` dispatches `OverlayState::drop` to the event
    // loop thread, which calls `DestroyWindow` and releases GDI objects.
    Ok(())
  }
}

/// Blits the stored screenshot (scaled to `rect`) into the layered overlay
/// window via `UpdateLayeredWindow`.
///
/// Must be called on the event loop thread (guaranteed by
/// `ThreadBound::with`).
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
