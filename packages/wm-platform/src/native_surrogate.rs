use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{BOOL, COLORREF, HWND, LPARAM, LRESULT, POINT, SIZE, WPARAM},
    Graphics::{
      Dwm::{DwmEnableBlurBehindWindow, DWM_BB_ENABLE, DWM_BLURBEHIND},
      Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateDIBSection,
        DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, BLENDFUNCTION, DIB_RGB_COLORS, HDC, HGDIOBJ,
        HRGN, SRCCOPY,
      },
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

/// Lightweight overlay window used during move/resize animations.
///
/// At animation start, the app window's on-screen content is captured into a
/// 32-bit BGRA frame buffer. Screenshot pixels are fully opaque (alpha = 255);
/// all other pixels are fully transparent (alpha = 0). `DwmEnableBlurBehindWindow`
/// is applied so transparent regions show a DWM blur of whatever is behind the
/// overlay — giving a Hyprland-style effect where the window content stays
/// visible at its captured size and the expanding region shows a blurred
/// background.
///
/// The frame buffer is pre-allocated at `max(source, target)` size in
/// [`NativeSurrogate::create`] so [`NativeSurrogate::update`] never allocates
/// GDI objects at runtime.
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
  /// Pixel dimensions of the captured snapshot.
  capture_width: i32,
  capture_height: i32,

  // --- Reusable frame buffer (written each frame) ---
  /// Memory DC used to compose each animation frame.
  frame_dc: isize,
  /// 32-bit BGRA `DIBSection` selected into `frame_dc`.
  frame_bitmap: isize,
  default_frame_bitmap: isize,
  /// Direct pointer to the `DIBSection` pixel data.
  ///
  /// Owned by GDI; invalidated when `frame_bitmap` is deleted.
  frame_bits: isize,
  /// Pre-allocated dimensions of `frame_bitmap`.
  frame_width: i32,
  frame_height: i32,
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

    // --- Screen DC ---
    //
    // SAFETY: `GetDC(HWND(0))` returns the screen DC; valid until released.
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

    // --- Frame DC + 32-bit DIBSection ---
    //
    // A `DIBSection` (top-down, 32bpp BGRA) is used so we can access the pixel
    // data directly. This lets `render_frame_buffer` set alpha=255 for the
    // screenshot region and leave alpha=0 (transparent) everywhere else.

    // SAFETY: `screen_dc` is valid.
    let frame_dc = unsafe { CreateCompatibleDC(screen_dc) };
    if frame_dc.0 == 0 {
      unsafe {
        ReleaseDC(HWND(0), screen_dc);
        SelectObject(capture_dc, default_capture_bitmap);
        DeleteObject(HGDIOBJ(capture_bitmap.0));
        DeleteDC(capture_dc);
      }
      return Err(crate::Error::Platform(
        "Failed to create frame DC.".to_string(),
      ));
    }

    let mut frame_bits: *mut std::ffi::c_void = std::ptr::null_mut();
    let bmi = Self::make_bitmapinfo(frame_w, frame_h);

    // SAFETY: `screen_dc` is valid; `bmi` is a correctly-initialized
    // `BITMAPINFO`; `frame_bits` receives a pointer to the pixel data.
    let frame_bitmap = unsafe {
      CreateDIBSection(
        screen_dc,
        &bmi,
        DIB_RGB_COLORS,
        &mut frame_bits,
        None,
        0,
      )
    };
    unsafe { ReleaseDC(HWND(0), screen_dc) };

    let frame_bitmap = match frame_bitmap {
      Ok(bmp) if bmp.0 != 0 => bmp,
      _ => {
        unsafe {
          SelectObject(capture_dc, default_capture_bitmap);
          DeleteObject(HGDIOBJ(capture_bitmap.0));
          DeleteDC(capture_dc);
          DeleteDC(frame_dc);
        }
        return Err(crate::Error::Platform(
          "Failed to create frame DIBSection.".to_string(),
        ));
      }
    };

    // SAFETY: Objects are valid.
    let default_frame_bitmap =
      unsafe { SelectObject(frame_dc, HGDIOBJ(frame_bitmap.0)) };

    // --- Screen capture ---
    //
    // Read from the DWM-composited screen at the source window's position.
    // SAFETY: `screen_dc` was released above; re-acquire for the capture.
    let screen_dc = unsafe { GetDC(HWND(0)) };

    // SAFETY: `screen_dc` is valid; coordinates are in screen space.
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

    unsafe { ReleaseDC(HWND(0), screen_dc) };

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
      }
      return Err(crate::Error::Platform(
        "Failed to create surrogate window.".to_string(),
      ));
    }

    let surrogate = Self {
      hwnd: hwnd.0,
      capture_dc: capture_dc.0,
      capture_bitmap: capture_bitmap.0,
      default_capture_bitmap: default_capture_bitmap.0,
      capture_width: src_w,
      capture_height: src_h,
      frame_dc: frame_dc.0,
      frame_bitmap: frame_bitmap.0,
      default_frame_bitmap: default_frame_bitmap.0,
      frame_bits: frame_bits as isize,
      frame_width: frame_w,
      frame_height: frame_h,
    };

    // Pre-render the frame buffer once. `update` only calls
    // `UpdateLayeredWindow` from this point on — no per-frame GDI blits.
    surrogate.render_frame_buffer();

    // Paint the initial frame before making the window visible.
    surrogate.apply_update(source_rect)?;

    // Enable DWM blur behind the layered window. Transparent pixels (alpha=0)
    // in the frame buffer will show a blurred view of whatever is behind the
    // surrogate. Screenshot pixels (alpha=255) show the frozen content.
    //
    // SAFETY: `hwnd` is a valid layered window.
    let blur_behind = DWM_BLURBEHIND {
      dwFlags: DWM_BB_ENABLE,
      fEnable: BOOL(1),
      hRgnBlur: HRGN(0), // Entire window region.
      fTransitionOnMaximized: BOOL(0),
    };
    let _ = unsafe { DwmEnableBlurBehindWindow(hwnd, &blur_behind) };

    // Place the surrogate immediately above `source_hwnd` in Z-order and
    // show it without activating it.
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

  /// Moves and resizes the surrogate overlay to `rect`.
  ///
  /// The frame buffer content was rendered once in [`NativeSurrogate::create`]
  /// (or after a rare buffer expansion). This method only calls
  /// `UpdateLayeredWindow` — no per-frame GDI pixel copies.
  pub fn update(&mut self, rect: &Rect) -> crate::Result<()> {
    // Expand pre-allocated buffer if needed (cancel-and-replace to a larger
    // target). This re-renders the frame buffer content automatically.
    if rect.width() > self.frame_width || rect.height() > self.frame_height {
      if let Err(err) =
        self.expand_frame_buffer(rect.width(), rect.height())
      {
        tracing::warn!(
          "Surrogate frame buffer too small and reallocation failed: {err}. \
           Clamping to pre-allocated size."
        );
      }
    }

    self.apply_update(rect)
  }

  /// Builds the `BITMAPINFO` descriptor for a top-down 32bpp BGRA `DIBSection`.
  fn make_bitmapinfo(width: i32, height: i32) -> BITMAPINFO {
    BITMAPINFO {
      bmiHeader: BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        // Negative height = top-down bitmap, matching screen coordinates.
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
      },
      ..Default::default()
    }
  }

  /// Writes the frame buffer content: screenshot pixels are fully opaque,
  /// background pixels are fully transparent (alpha = 0 → shows DWM blur).
  ///
  /// Called once at creation and once after each buffer expansion. The result
  /// persists for the entire animation; [`NativeSurrogate::update`] reads from
  /// this buffer every frame without re-rendering it.
  fn render_frame_buffer(&self) {
    // Copy the screenshot into the frame DC. `BitBlt` writes RGB values but
    // leaves the alpha channel at 0 in the DIBSection.
    let copy_w = self.capture_width.min(self.frame_width);
    let copy_h = self.capture_height.min(self.frame_height);

    if copy_w > 0 && copy_h > 0 {
      // SAFETY: All DC/bitmap handles are valid; dimensions are positive.
      let _ = unsafe {
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
      };
    }

    // Set alpha=255 for screenshot pixels so they are fully opaque.
    // Background pixels remain at alpha=0 (transparent), which lets
    // `DwmEnableBlurBehindWindow` show the blurred desktop behind them.
    //
    // SAFETY: `frame_bits` points to the `DIBSection` pixel array; the loop
    // stays within `[0, copy_w) × [0, copy_h)`, well within the allocated
    // `frame_width × frame_height` buffer.
    let bits = self.frame_bits as *mut u32;
    let stride = self.frame_width as usize;
    for y in 0..copy_h as usize {
      for x in 0..copy_w as usize {
        unsafe { *bits.add(y * stride + x) |= 0xFF00_0000 };
      }
    }
  }

  /// Submits the current frame buffer to DWM at the given position/size.
  fn apply_update(&self, rect: &Rect) -> crate::Result<()> {
    let draw_w = rect.width().min(self.frame_width).max(0);
    let draw_h = rect.height().min(self.frame_height).max(0);

    if draw_w == 0 || draw_h == 0 {
      return Ok(());
    }

    // Per-pixel alpha: screenshot region is opaque (alpha=255), background
    // region is transparent (alpha=0) and shows the DWM blur behind.
    let blend = BLENDFUNCTION {
      BlendOp: 0,    // AC_SRC_OVER
      BlendFlags: 0,
      SourceConstantAlpha: 255,
      AlphaFormat: 1, // AC_SRC_ALPHA — use per-pixel alpha from DIBSection
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

    // SAFETY: `HWND(self.hwnd)` is a valid layered window; all structs are
    // properly initialized.
    let screen_dc = unsafe { GetDC(HWND(0)) };
    let result = unsafe {
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

    result?;
    Ok(())
  }

  /// Expands the frame buffer to at least `needed_w × needed_h`.
  ///
  /// Called at most once per animation (only when cancel-and-replace targets
  /// a size larger than the original pre-allocation).
  fn expand_frame_buffer(
    &mut self,
    needed_w: i32,
    needed_h: i32,
  ) -> crate::Result<()> {
    let new_w = needed_w.max(self.frame_width);
    let new_h = needed_h.max(self.frame_height);

    let mut new_bits: *mut std::ffi::c_void = std::ptr::null_mut();
    let bmi = Self::make_bitmapinfo(new_w, new_h);

    // SAFETY: `frame_dc` is valid; `bmi` is correctly initialized.
    let screen_dc = unsafe { GetDC(HWND(0)) };
    let new_bitmap = unsafe {
      CreateDIBSection(
        screen_dc,
        &bmi,
        DIB_RGB_COLORS,
        &mut new_bits,
        None,
        0,
      )
    };
    unsafe { ReleaseDC(HWND(0), screen_dc) };

    let new_bitmap = new_bitmap.map_err(|e| {
      crate::Error::Platform(format!(
        "Failed to reallocate frame DIBSection: {e}."
      ))
    })?;

    // Swap the bitmap in the frame DC.
    //
    // SAFETY: `frame_dc` is valid; objects are valid GDI handles.
    unsafe {
      SelectObject(HDC(self.frame_dc), HGDIOBJ(self.default_frame_bitmap));
      DeleteObject(HGDIOBJ(self.frame_bitmap));
      SelectObject(HDC(self.frame_dc), HGDIOBJ(new_bitmap.0));
    }

    self.frame_bitmap = new_bitmap.0;
    self.frame_bits = new_bits as isize;
    self.frame_width = new_w;
    self.frame_height = new_h;

    // Re-render into the newly allocated buffer.
    self.render_frame_buffer();

    Ok(())
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

      let _ = DestroyWindow(HWND(self.hwnd));
    }
  }
}
