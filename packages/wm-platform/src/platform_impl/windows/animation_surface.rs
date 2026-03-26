use std::{
  cell::{Cell, RefCell},
  collections::HashMap,
  sync::OnceLock,
};

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

use crate::{
  animation_surface::LayerId, Dispatcher, NativeWindow,
  NativeWindowWindowsExt, OpacityValue, Rect,
};

/// Guard ensuring the overlay window class is registered at most once per
/// process.
static OVERLAY_CLASS: OnceLock<()> = OnceLock::new();

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
  /// Current constant opacity (0-255). Uses `Cell` for interior
  /// mutability.
  alpha: Cell<u8>,
  /// Current overlay position and size. Uses `RefCell` for interior
  /// mutability.
  current_rect: RefCell<Rect>,
}

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
struct OverlayWindow {
  inner: OverlayState,
}

// SAFETY: The `HWND`, `HDC`, and `HGDIOBJ` handles are only accessed from
// the thread that created them via `update_layered`.
unsafe impl Send for OverlayWindow {}
// SAFETY: Interior-mutable fields use `Cell`/`RefCell` for thread-local
// access only.
unsafe impl Sync for OverlayWindow {}

impl OverlayWindow {
  /// Creates a layered overlay `HWND` at `initial_rect` using
  /// pre-captured GDI handles `(hdc_mem, hbitmap)`.
  fn new(
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

  /// Moves and resizes the overlay to match `rect`.
  fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    let rect = rect.clone();
    self.inner.current_rect.replace(rect.clone());
    update_layered(&self.inner, &rect);
    Ok(())
  }

  /// Sets overlay opacity (0.0-1.0). For fade animations.
  fn set_opacity(&self, alpha: f32) -> crate::Result<()> {
    let alpha = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    self.inner.alpha.set(alpha);
    let rect = self.inner.current_rect.borrow().clone();
    update_layered(&self.inner, &rect);
    Ok(())
  }

  /// Destroys the overlay window.
  fn destroy(self) -> crate::Result<()> {
    // TODO: Destroy the window.
    unsafe { ShowWindow(HWND(self.inner.hwnd), SW_HIDE) }?;
    Ok(())
  }

  /// Window procedure for the overlay class.
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

/// Platform-specific implementation of [`AnimationSurface`].
pub(crate) struct AnimationSurface {
  layers: HashMap<LayerId, OverlayWindow>,
  next_id: u64,
  dispatcher: Dispatcher,
}

impl AnimationSurface {
  /// Implements [`AnimationSurface::new`].
  pub(crate) fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    Ok(Self {
      layers: HashMap::new(),
      next_id: 0,
      dispatcher: dispatcher.clone(),
    })
  }

  /// Implements [`AnimationSurface::add_layer`].
  pub(crate) fn add_layer(
    &mut self,
    window: &NativeWindow,
    rect: &Rect,
    opacity: Option<f32>,
  ) -> crate::Result<LayerId> {
    let capture = window.screen_capture(rect)?;
    let overlay = OverlayWindow::new(capture, rect)?;

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

  /// Implements [`AnimationSurface::update_layers`].
  pub(crate) fn update_layers(
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

  /// Implements [`AnimationSurface::remove_layer`].
  pub(crate) fn remove_layer(&mut self, id: LayerId) -> crate::Result<()> {
    if let Some(overlay) = self.layers.remove(&id) {
      if let Err(err) = overlay.destroy() {
        tracing::warn!("Failed to destroy overlay layer: {}", err);
      }
    }

    Ok(())
  }

  /// Implements [`AnimationSurface::has_layers`].
  pub(crate) fn has_layers(&self) -> crate::Result<bool> {
    Ok(!self.layers.is_empty())
  }

  /// No-op on Windows — each layer's `HWND` is shown on creation.
  pub(crate) fn show(&self) -> crate::Result<()> {
    Ok(())
  }

  /// No-op on Windows — layers are destroyed individually via
  /// `remove_layer`.
  pub(crate) fn hide(&self) -> crate::Result<()> {
    Ok(())
  }

  /// Implements [`AnimationSurface::destroy`].
  pub(crate) fn destroy(mut self) -> crate::Result<()> {
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
