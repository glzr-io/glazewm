use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM},
    Graphics::Gdi::{
      BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush,
      DeleteDC, DeleteObject, FillRect, GetDC, GetPixel, ReleaseDC,
      SelectObject, BLENDFUNCTION, HBRUSH, HDC, HGDIOBJ, SRCCOPY,
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW,
      SetWindowPos, UpdateLayeredWindow, SWP_NOACTIVATE, SWP_NOMOVE,
      SWP_NOSIZE, SWP_SHOWWINDOW, ULW_ALPHA, WNDCLASSW, WS_EX_LAYERED,
      WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::Rect;

/// Ensures the surrogate window class is registered exactly once per process.
static SURROGATE_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// Default window procedure wrapper with the required `extern "system"` ABI.
///
/// `DefWindowProcW` in windows-rs is generic and cannot be coerced to a
/// bare function pointer directly.
unsafe extern "system" fn default_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // SAFETY: All parameters are passed through unchanged.
  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn ensure_class_registered() {
  SURROGATE_CLASS_REGISTERED.get_or_init(|| {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_Surrogate"),
      lpfnWndProc: Some(default_wnd_proc),
      ..Default::default()
    };

    // SAFETY: `wnd_class` is a properly initialized `WNDCLASSW` with a
    // static class name and a valid window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
}

/// Lightweight overlay window used to display a frozen snapshot of an app
/// window during move/resize animations.
///
/// The surrogate sits on top of the real app window and shows what the app
/// looked like before the animation started. GlazeWM animates only this
/// overlay every frame while the real app receives no resize messages.
/// When the animation finishes the real window is moved to its final
/// position and the surrogate is dropped.
///
/// To avoid per-frame GDI allocation (the primary source of lag), the
/// frame buffer is pre-allocated at `max(source, target)` size in
/// [`NativeSurrogate::create`] and reused for every [`NativeSurrogate::update`]
/// call.
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeSurrogate {
  /// Handle to the overlay window.
  hwnd: isize,

  // --- Captured snapshot (read-only after creation) ---
  /// Memory DC holding the captured source bitmap.
  capture_dc: isize,
  capture_bitmap: isize,
  default_capture_bitmap: isize,
  /// Width of the captured snapshot in pixels.
  capture_width: i32,
  /// Height of the captured snapshot in pixels.
  capture_height: i32,

  // --- Reusable frame buffer (written each frame) ---
  /// Memory DC used to compose each animation frame.
  frame_dc: isize,
  /// Bitmap selected into `frame_dc`; pre-allocated at the maximum size
  /// required across the entire animation so no allocation happens per frame.
  frame_bitmap: isize,
  default_frame_bitmap: isize,
  /// Pre-allocated dimensions of `frame_bitmap`.
  frame_width: i32,
  frame_height: i32,

  /// Pre-created solid-color brush used to fill areas of the frame not
  /// covered by the captured snapshot.
  background_brush: isize,

  /// Position of the real app window at the moment the animation started.
  /// The real window is kept at this rect for the entire animation so that
  /// it never receives intermediate resize messages.
  pub frozen_rect: Rect,
}

impl NativeSurrogate {
  /// Creates a surrogate by capturing the on-screen content of `source_hwnd`.
  ///
  /// The frame buffer is pre-allocated at
  /// `max(source_rect.width(), target_rect.width()) ×
  ///  max(source_rect.height(), target_rect.height())`
  /// so that [`NativeSurrogate::update`] never needs to allocate GDI objects
  /// at runtime.
  ///
  /// Returns an error if window or GDI resource creation fails.
  pub fn create(
    source_hwnd: HWND,
    source_rect: &Rect,
    target_rect: &Rect,
  ) -> crate::Result<Self> {
    ensure_class_registered();

    let src_w = source_rect.width();
    let src_h = source_rect.height();

    if src_w <= 0 || src_h <= 0 {
      return Err(crate::Error::Platform(
        "Surrogate source rect has zero or negative dimensions.".to_string(),
      ));
    }

    // Pre-allocate the frame buffer large enough for the entire animation so
    // that no allocations are needed per frame.
    let frame_w = src_w.max(target_rect.width()).max(1);
    let frame_h = src_h.max(target_rect.height()).max(1);

    // --- Screen DC (needed to create compatible bitmaps) ---
    //
    // SAFETY: `GetDC(HWND(0))` returns the screen DC; valid until
    // released with `ReleaseDC`.
    let screen_dc = unsafe { GetDC(HWND(0)) };

    // --- Capture DC + bitmap ---
    //
    // SAFETY: `screen_dc` is a valid DC.
    let capture_dc = unsafe { CreateCompatibleDC(screen_dc) };
    if capture_dc.0 == 0 {
      unsafe { ReleaseDC(HWND(0), screen_dc) };
      return Err(crate::Error::Platform(
        "Failed to create capture DC.".to_string(),
      ));
    }

    // SAFETY: `screen_dc` is valid and dimensions are positive.
    let capture_bitmap =
      unsafe { CreateCompatibleBitmap(screen_dc, src_w, src_h) };
    if capture_bitmap.0 == 0 {
      unsafe {
        ReleaseDC(HWND(0), screen_dc);
        DeleteDC(capture_dc);
      }
      return Err(crate::Error::Platform(
        "Failed to create capture bitmap.".to_string(),
      ));
    }

    // SAFETY: Objects are valid.
    let default_capture_bitmap =
      unsafe { SelectObject(capture_dc, HGDIOBJ(capture_bitmap.0)) };

    // --- Frame DC + bitmap ---
    //
    // SAFETY: `screen_dc` is valid and dimensions are positive.
    let frame_dc = unsafe { CreateCompatibleDC(screen_dc) };
    let frame_bitmap =
      unsafe { CreateCompatibleBitmap(screen_dc, frame_w, frame_h) };

    if frame_dc.0 == 0 || frame_bitmap.0 == 0 {
      unsafe {
        ReleaseDC(HWND(0), screen_dc);
        SelectObject(capture_dc, default_capture_bitmap);
        DeleteObject(HGDIOBJ(capture_bitmap.0));
        DeleteDC(capture_dc);
        if frame_dc.0 != 0 {
          DeleteDC(frame_dc);
        }
        if frame_bitmap.0 != 0 {
          DeleteObject(HGDIOBJ(frame_bitmap.0));
        }
      }
      return Err(crate::Error::Platform(
        "Failed to create frame DC/bitmap.".to_string(),
      ));
    }

    // SAFETY: Objects are valid.
    let default_frame_bitmap =
      unsafe { SelectObject(frame_dc, HGDIOBJ(frame_bitmap.0)) };

    // --- Screen capture ---
    //
    // Read from the DWM-composited screen at the source window's position.
    // This captures all rendering technologies (GDI, Direct2D, DirectX,
    // WebGL) because it reads the compositor output rather than asking the
    // window to repaint. `PrintWindow` would be preferable for occluded
    // windows, but it is not exposed in windows-rs 0.52.
    //
    // SAFETY: `screen_dc` is valid; coordinates are in screen space;
    // `capture_dc` has an appropriately sized bitmap selected.
    let _ = unsafe {
      BitBlt(
        capture_dc,
        0,
        0,
        src_w,
        src_h,
        screen_dc,
        source_rect.x(),
        source_rect.y(),
        SRCCOPY,
      )
    };

    let background_color =
      Self::sample_background_color(capture_dc, src_w, src_h);

    unsafe { ReleaseDC(HWND(0), screen_dc) };

    // --- Background brush (reused every frame) ---
    //
    // SAFETY: `CreateSolidBrush` is safe for any valid `COLORREF`.
    let background_brush =
      unsafe { CreateSolidBrush(COLORREF(background_color)) };

    // --- Surrogate window ---
    //
    // Layered, non-activating, taskbar-invisible, mouse-transparent pop-up.
    let ex_style = WS_EX_LAYERED
      | WS_EX_NOACTIVATE
      | WS_EX_TOOLWINDOW
      | WS_EX_TRANSPARENT;

    // SAFETY: Class name is the static literal registered above.
    let hwnd = unsafe {
      CreateWindowExW(
        ex_style,
        w!("GlazeWM_Surrogate"),
        w!(""),
        WS_POPUP,
        source_rect.x(),
        source_rect.y(),
        src_w,
        src_h,
        None,
        None,
        None,
        None,
      )
    };

    if hwnd.0 == 0 {
      unsafe {
        SelectObject(capture_dc, default_capture_bitmap);
        DeleteObject(HGDIOBJ(capture_bitmap.0));
        DeleteDC(capture_dc);
        SelectObject(frame_dc, default_frame_bitmap);
        DeleteObject(HGDIOBJ(frame_bitmap.0));
        DeleteDC(frame_dc);
        DeleteObject(background_brush);
      }
      return Err(crate::Error::Platform(
        "Failed to create surrogate window.".to_string(),
      ));
    }

    let mut surrogate = Self {
      hwnd: hwnd.0,
      capture_dc: capture_dc.0,
      capture_bitmap: capture_bitmap.0,
      default_capture_bitmap: default_capture_bitmap.0,
      capture_width: src_w,
      capture_height: src_h,
      frame_dc: frame_dc.0,
      frame_bitmap: frame_bitmap.0,
      default_frame_bitmap: default_frame_bitmap.0,
      frame_width: frame_w,
      frame_height: frame_h,
      background_brush: background_brush.0,
      frozen_rect: source_rect.clone(),
    };

    // Paint the initial frame before making the window visible.
    surrogate.update(source_rect)?;

    // Place the surrogate immediately above `source_hwnd` in the Z-order
    // and show it without activating it.
    //
    // SAFETY: Both handles are valid.
    unsafe {
      SetWindowPos(
        HWND(surrogate.hwnd),
        source_hwnd,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
      )
    }?;

    Ok(surrogate)
  }

  /// Updates the surrogate's position and size, recompositing the captured
  /// snapshot into the pre-allocated frame buffer.
  ///
  /// The captured snapshot is drawn at its natural size from the top-left
  /// corner. Any area beyond the snapshot dimensions is filled with the
  /// background color. `UpdateLayeredWindow` repositions and repaints the
  /// overlay in a single atomic DWM call with no GDI allocations.
  pub fn update(&mut self, rect: &Rect) -> crate::Result<()> {
    let new_w = rect.width();
    let new_h = rect.height();

    if new_w <= 0 || new_h <= 0 {
      return Ok(());
    }

    // If the visible size exceeds the pre-allocated frame buffer (can only
    // happen on cancel-and-replace to a significantly larger target), expand
    // it. This is a rare path and involves one reallocation.
    if new_w > self.frame_width || new_h > self.frame_height {
      if let Err(err) = self.expand_frame_buffer(new_w, new_h) {
        tracing::warn!(
          "Surrogate frame buffer too small and reallocation failed: {err}. \
           Clamping to pre-allocated size."
        );
      }
    }

    let draw_w = new_w.min(self.frame_width);
    let draw_h = new_h.min(self.frame_height);

    // Fill the visible area with the background color to clear any
    // leftover content from a previous (larger) frame.
    let fill = RECT {
      left: 0,
      top: 0,
      right: draw_w,
      bottom: draw_h,
    };
    // SAFETY: `frame_dc` is a valid DC with `frame_bitmap` selected;
    // `background_brush` is a valid pre-created solid brush.
    unsafe { FillRect(HDC(self.frame_dc), &fill, HBRUSH(self.background_brush)) };

    // Overlay the captured snapshot, clipped to the smaller of capture and
    // visible dimensions.
    let copy_w = self.capture_width.min(draw_w);
    let copy_h = self.capture_height.min(draw_h);

    if copy_w > 0 && copy_h > 0 {
      // SAFETY: All DC/bitmap objects are valid; dimensions are positive.
      unsafe {
        BitBlt(
          HDC(self.frame_dc),
          0,
          0,
          copy_w,
          copy_h,
          HDC(self.capture_dc),
          0,
          0,
          SRCCOPY,
        )
      }?;
    }

    // `UpdateLayeredWindow` repositions the overlay and applies the new
    // frame content atomically. `ULW_ALPHA` with `SourceConstantAlpha = 255`
    // and `AlphaFormat = 0` renders the window fully opaque (no per-pixel
    // alpha channel required in the source bitmap).
    let blend = BLENDFUNCTION {
      BlendOp: 0, // AC_SRC_OVER
      BlendFlags: 0,
      SourceConstantAlpha: 255,
      AlphaFormat: 0,
    };
    let pt_src = POINT { x: 0, y: 0 };
    let pt_dst = POINT {
      x: rect.x(),
      y: rect.y(),
    };
    let sz = SIZE {
      cx: draw_w,
      cy: draw_h,
    };

    // SAFETY: `HWND(self.hwnd)` is a valid layered window; `screen_dc` and
    // all structs are properly initialized.
    let screen_dc = unsafe { GetDC(HWND(0)) };
    let update_result = unsafe {
      UpdateLayeredWindow(
        HWND(self.hwnd),
        screen_dc,
        Some(&raw const pt_dst),
        Some(&raw const sz),
        HDC(self.frame_dc),
        Some(&raw const pt_src),
        COLORREF(0),
        Some(&raw const blend),
        ULW_ALPHA,
      )
    };
    unsafe { ReleaseDC(HWND(0), screen_dc) };

    update_result?;
    Ok(())
  }

  /// Expands the frame buffer to at least `needed_w × needed_h`.
  ///
  /// Called at most once per animation (only when cancel-and-replace
  /// targets a size larger than the original pre-allocation).
  fn expand_frame_buffer(
    &mut self,
    needed_w: i32,
    needed_h: i32,
  ) -> crate::Result<()> {
    let new_w = needed_w.max(self.frame_width);
    let new_h = needed_h.max(self.frame_height);

    // SAFETY: `GetDC(HWND(0))` returns the screen DC.
    let screen_dc = unsafe { GetDC(HWND(0)) };
    // SAFETY: `screen_dc` is valid and dimensions are positive.
    let new_bitmap =
      unsafe { CreateCompatibleBitmap(screen_dc, new_w, new_h) };
    unsafe { ReleaseDC(HWND(0), screen_dc) };

    if new_bitmap.0 == 0 {
      return Err(crate::Error::Platform(
        "Failed to reallocate frame bitmap.".to_string(),
      ));
    }

    // Swap the bitmap in the frame DC.
    //
    // SAFETY: `frame_dc` is valid; objects are valid GDI handles.
    unsafe {
      SelectObject(HDC(self.frame_dc), HGDIOBJ(self.default_frame_bitmap));
      DeleteObject(HGDIOBJ(self.frame_bitmap));
      SelectObject(HDC(self.frame_dc), HGDIOBJ(new_bitmap.0));
    }

    self.frame_bitmap = new_bitmap.0;
    self.frame_width = new_w;
    self.frame_height = new_h;

    Ok(())
  }

  /// Samples a representative background color from the captured bitmap.
  ///
  /// Reads pixels at a 3×3 grid, filters out `CLR_INVALID` results, and
  /// returns the median value. Falls back to black if all samples fail.
  fn sample_background_color(
    capture_dc: HDC,
    width: i32,
    height: i32,
  ) -> u32 {
    let margin = (width.min(height) / 10).max(2);
    let mid_x = width / 2;
    let mid_y = height / 2;

    let positions = [
      (margin, margin),
      (mid_x, margin),
      (width - margin, margin),
      (margin, mid_y),
      (mid_x, mid_y),
      (width - margin, mid_y),
      (margin, height - margin),
      (mid_x, height - margin),
      (width - margin, height - margin),
    ];

    let mut colors: Vec<u32> = positions
      .iter()
      .map(|(x, y)| {
        // SAFETY: `capture_dc` is a valid memory DC with a bitmap selected.
        unsafe { GetPixel(capture_dc, *x, *y).0 }
      })
      .filter(|&c| c != 0xFFFF_FFFF) // CLR_INVALID
      .collect();

    if colors.is_empty() {
      return 0; // Fallback to black.
    }

    colors.sort_unstable();
    colors[colors.len() / 2]
  }
}

impl Drop for NativeSurrogate {
  fn drop(&mut self) {
    // SAFETY: All handles were obtained in `create` and remain valid.
    // GDI objects must be de-selected before deletion to avoid leaks.
    unsafe {
      SelectObject(HDC(self.capture_dc), HGDIOBJ(self.default_capture_bitmap));
      DeleteObject(HGDIOBJ(self.capture_bitmap));
      DeleteDC(HDC(self.capture_dc));

      SelectObject(HDC(self.frame_dc), HGDIOBJ(self.default_frame_bitmap));
      DeleteObject(HGDIOBJ(self.frame_bitmap));
      DeleteDC(HDC(self.frame_dc));

      DeleteObject(HGDIOBJ(self.background_brush));

      let _ = DestroyWindow(HWND(self.hwnd));
    }
  }
}
