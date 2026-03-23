#[cfg(target_os = "macos")]
use objc2::{AnyThread, MainThreadMarker, MainThreadOnly};
#[cfg(target_os = "macos")]
use objc2_app_kit::{
  NSBackingStoreType, NSColor, NSFloatingWindowLevel, NSImage,
  NSImageView, NSScreen, NSWindow, NSWindowStyleMask,
};
#[cfg(target_os = "macos")]
use objc2_core_foundation::{CGRect, CGSize};
#[cfg(target_os = "macos")]
#[allow(deprecated)]
use objc2_core_graphics::{
  CGAffineTransformConcat, CGAffineTransformMakeScale,
  CGAffineTransformMakeTranslation, CGWindowImageOption,
  CGWindowListCreateImage, CGWindowListOption,
};
#[cfg(target_os = "macos")]
use objc2_foundation::NSRect;

#[cfg(target_os = "macos")]
use crate::platform_impl::ffi;
use crate::OpacityValue;
#[cfg(target_os = "macos")]
use crate::{Dispatcher, Rect, ThreadBound, WindowId};

/// Batches frame and opacity updates for multiple overlays into a single
/// SLS transaction.
#[cfg(target_os = "macos")]
pub fn move_group(
  overlays: &[(&OverlayWindow, &Rect, Option<OpacityValue>)],
) {
  let cid = unsafe { ffi::SLSMainConnectionID() };
  unsafe { ffi::SLSDisableUpdate(cid) };

  let txn = unsafe { ffi::SLSTransactionCreate(cid) };

  if txn.is_null() {
    return;
  }

  for (overlay, rect, opacity) in overlays {
    overlay.apply_transform(rect, txn);

    if let Some(opacity) = opacity {
      overlay.apply_alpha(opacity.to_f32(), txn);
    }
  }

  // Single commit for the entire batch.
  unsafe { ffi::SLSTransactionCommit(txn, 0) };
  unsafe { ffi::SLSReenableUpdate(cid) };
}

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

/// A borderless overlay `NSWindow` displaying a screenshot of a real
/// window.
///
/// Used for smooth animations — moving our own window is much cheaper than
/// AX API calls on 3rd-party windows.
#[cfg(target_os = "macos")]
pub struct OverlayWindow {
  ns_window: ThreadBound<objc2::rc::Retained<NSWindow>>,
  /// CGWindowID of the overlay `NSWindow`, used by SLS transactions.
  window_id: u32,
  /// Initial frame the window was created at, used as the reference
  /// point for SLS affine transforms.
  initial_rect: Rect,
}

#[cfg(target_os = "macos")]
impl OverlayWindow {
  /// Screenshots the window and creates an overlay `NSWindow` at
  /// `initial_rect`.
  #[allow(deprecated)] // CGWindowListCreateImage is deprecated but functional.
  pub fn new(
    window_id: WindowId,
    initial_rect: &Rect,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let wid = window_id.0;
    let rect = initial_rect.clone();
    let disp = dispatcher.clone();

    let (ns_window, window_id) = dispatcher.dispatch_sync(move || {
      // SAFETY: `dispatch_sync` executes on the event loop (main) thread.
      let mtm = unsafe { MainThreadMarker::new_unchecked() };

      // Screenshot the target window.
      let cg_rect = CGRect::new(
        objc2_core_foundation::CGPoint {
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

      let ns_rect = NSRect::new(
        objc2_foundation::NSPoint {
          x: f64::from(rect.x()),
          y: flipped_y(&rect, mtm),
        },
        objc2_foundation::NSSize {
          width: f64::from(rect.width()),
          height: f64::from(rect.height()),
        },
      );

      // Create borderless NSWindow.
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

      // SAFETY: We own this window and manage its lifetime via
      // `ThreadBound` + `orderOut`.
      unsafe { window.setReleasedWhenClosed(false) };

      // Build the image content from the screenshot.
      if let Some(cg_image) = cg_image {
        let logical_size = CGSize {
          width: f64::from(rect.width()),
          height: f64::from(rect.height()),
        };
        let ns_image = NSImage::initWithCGImage_size(
          NSImage::alloc(),
          &cg_image,
          logical_size,
        );

        let image_view = NSImageView::imageViewWithImage(&ns_image, mtm);
        window.setContentView(Some(&image_view));
      }

      window.orderFrontRegardless();

      let wid = window.windowNumber() as u32;

      (ThreadBound::new(window, disp), wid)
    })?;

    Ok(Self {
      ns_window,
      window_id,
      initial_rect: initial_rect.clone(),
    })
  }

  /// Moves and scales the overlay via an SLS affine transform relative to
  /// the initial frame.
  ///
  /// Creates a dedicated transaction, applies the transform, and commits.
  /// For batched updates, use `move_group` instead.
  pub fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    let cid = unsafe { ffi::SLSMainConnectionID() };
    let txn = unsafe { ffi::SLSTransactionCreate(cid) };

    if txn.is_null() {
      return Ok(());
    }

    self.apply_transform(rect, txn);
    unsafe { ffi::SLSTransactionCommit(txn, 0) };

    Ok(())
  }

  /// Sets overlay opacity (0.0–1.0) via SLS transaction.
  ///
  /// Creates a dedicated transaction. For batched updates, use
  /// `move_group` instead.
  pub fn set_opacity(&self, alpha: f32) -> crate::Result<()> {
    let cid = unsafe { ffi::SLSMainConnectionID() };
    let txn = unsafe { ffi::SLSTransactionCreate(cid) };

    if txn.is_null() {
      return Ok(());
    }

    self.apply_alpha(alpha, txn);
    unsafe { ffi::SLSTransactionCommit(txn, 0) };

    Ok(())
  }

  /// Queues an affine transform into an existing transaction without
  /// committing.
  ///
  /// The SLS transform is an inverse mapping from screen coordinates to
  /// content coordinates: `translate(-target_x, -target_y)` shifts the
  /// screen origin, then `scale(init_w/target_w, init_h/target_h)` maps
  /// the target size back to the original content size.
  fn apply_transform(&self, rect: &Rect, txn: *mut c_void) {
    let init = &self.initial_rect;

    // Inverse mapping: screen → content coordinates.
    let translate = CGAffineTransformMakeTranslation(
      -f64::from(rect.x()),
      -f64::from(rect.y()),
    );
    let scale = CGAffineTransformMakeScale(
      f64::from(init.width()) / f64::from(rect.width()),
      f64::from(init.height()) / f64::from(rect.height()),
    );
    let transform = CGAffineTransformConcat(translate, scale);

    // SAFETY: SkyLight SPI calls are unsafe but stable in practice.
    unsafe {
      ffi::SLSTransactionSetWindowTransform(
        txn,
        self.window_id,
        0,
        0,
        transform,
      );
    }
  }

  /// Queues an alpha change into an existing transaction without
  /// committing.
  fn apply_alpha(&self, alpha: f32, txn: *mut c_void) {
    let _ = self.ns_window.with(|window| {
      window.setAlphaValue(f64::from(alpha));
    });
    // TODO: Setting alpha via SLS transaction doesn't seem to work.
    // unsafe {
    //   ffi::SLSTransactionSetWindowAlpha(
    //     txn,
    //     self.window_id,
    //     f64::from(alpha),
    //   );
    // }
  }

  /// Destroys the overlay window by ordering it out and dropping the
  /// handle.
  pub fn destroy(self) -> crate::Result<()> {
    self.ns_window.with(|window| {
      window.orderOut(None);
    })
  }
}

/// Converts a `Rect` top-left Y to the flipped `NSWindow` coordinate
/// system (origin at bottom-left of primary screen).
#[cfg(target_os = "macos")]
fn flipped_y(rect: &Rect, mtm: MainThreadMarker) -> f64 {
  let screen_height = NSScreen::screens(mtm)
    .into_iter()
    .next()
    .map_or(0.0, |s| s.frame().size.height);

  screen_height - f64::from(rect.y()) - f64::from(rect.height())
}

#[cfg(target_os = "macos")]
impl std::fmt::Debug for OverlayWindow {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OverlayWindow").finish_non_exhaustive()
  }
}

// ── Windows implementation
// ────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
use std::cell::{Cell, RefCell};
#[cfg(target_os = "macos")]
use std::ffi::c_void;
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
