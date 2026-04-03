use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM},
    Graphics::Gdi::{
      BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateSolidBrush,
      DeleteDC, DeleteObject, FillRect, GetDC, GetPixel, ReleaseDC,
      SelectObject, BLENDFUNCTION, HDC, HGDIOBJ, SRCCOPY,
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
///
/// SAFETY: Delegates unconditionally to the system `DefWindowProcW`.
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

    // SAFETY: `wnd_class` is a properly initialized `WNDCLASSW`. The class
    // name is a static wide string literal and remains valid indefinitely.
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
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeSurrogate {
  /// Handle to the overlay window.
  hwnd: isize,
  /// Memory DC containing the captured bitmap.
  capture_dc: isize,
  /// Bitmap capturing the source window's on-screen content.
  capture_bitmap: isize,
  /// Default 1×1 bitmap that was in `capture_dc` before `capture_bitmap`
  /// was selected into it; restored in `Drop` to allow clean deletion.
  default_bitmap: isize,
  /// Width of the captured content in pixels.
  capture_width: i32,
  /// Height of the captured content in pixels.
  capture_height: i32,
  /// Background color sampled from the source window, used to fill any
  /// area not covered by the captured snapshot (e.g. when the window
  /// grows beyond its original size during animation).
  background_color: u32,
  /// Position of the real app window at the moment the animation started.
  /// The real window is kept at this rect for the entire animation so that
  /// it never receives intermediate resize messages.
  pub frozen_rect: Rect,
}

impl NativeSurrogate {
  /// Creates a surrogate by capturing the on-screen content of `source_hwnd`.
  ///
  /// Copies the pixels currently visible at `source_rect` from the
  /// composited screen output into a frozen snapshot, creates a layered
  /// overlay window at the same position, and places it immediately above
  /// `source_hwnd` in the Z-order. Returns an error if window or GDI
  /// resource creation fails.
  pub fn create(
    source_hwnd: HWND,
    source_rect: &Rect,
  ) -> crate::Result<Self> {
    ensure_class_registered();

    let width = source_rect.width();
    let height = source_rect.height();

    if width <= 0 || height <= 0 {
      return Err(crate::Error::Platform(
        "Surrogate source rect has zero or negative dimensions.".to_string(),
      ));
    }

    // Capture visible screen content at the source window's location.
    // Reading from the screen DC works for all rendering technologies (GDI,
    // DirectX, WebGL) because it reads the DWM-composited output.
    //
    // SAFETY: `GetDC(HWND(0))` retrieves the screen DC; valid until
    // released with `ReleaseDC`.
    let screen_dc = unsafe { GetDC(HWND(0)) };

    // SAFETY: `screen_dc` is a valid DC.
    let capture_dc = unsafe { CreateCompatibleDC(screen_dc) };
    if capture_dc.0 == 0 {
      unsafe { ReleaseDC(HWND(0), screen_dc) };
      return Err(crate::Error::Platform(
        "Failed to create capture DC.".to_string(),
      ));
    }

    // SAFETY: `screen_dc` is valid and `width`/`height` are positive.
    let capture_bitmap =
      unsafe { CreateCompatibleBitmap(screen_dc, width, height) };
    if capture_bitmap.0 == 0 {
      unsafe {
        ReleaseDC(HWND(0), screen_dc);
        DeleteDC(capture_dc);
      }
      return Err(crate::Error::Platform(
        "Failed to create capture bitmap.".to_string(),
      ));
    }

    // Select the capture bitmap, saving the default 1×1 bitmap for
    // restoration in `Drop`.
    //
    // SAFETY: `capture_dc` and `capture_bitmap` are valid GDI objects.
    let default_bitmap =
      unsafe { SelectObject(capture_dc, HGDIOBJ(capture_bitmap.0)) };

    // SAFETY: `screen_dc` is valid; coordinates are in screen space;
    // `capture_dc` has an appropriately sized bitmap selected.
    let _ = unsafe {
      BitBlt(
        capture_dc,
        0,
        0,
        width,
        height,
        screen_dc,
        source_rect.x(),
        source_rect.y(),
        SRCCOPY,
      )
    };

    let background_color =
      Self::sample_background_color(capture_dc, width, height);

    unsafe { ReleaseDC(HWND(0), screen_dc) };

    // Create a layered, non-activating tool-window pop-up invisible to the
    // taskbar and immune to mouse/keyboard input (`WS_EX_TRANSPARENT`).
    let ex_style = WS_EX_LAYERED
      | WS_EX_NOACTIVATE
      | WS_EX_TOOLWINDOW
      | WS_EX_TRANSPARENT;

    // SAFETY: Class name is the static literal registered above.
    // `hInstance` defaults to the current module (null = current module).
    let hwnd = unsafe {
      CreateWindowExW(
        ex_style,
        w!("GlazeWM_Surrogate"),
        w!(""),
        WS_POPUP,
        source_rect.x(),
        source_rect.y(),
        width,
        height,
        None,
        None,
        None,
        None,
      )
    };

    if hwnd.0 == 0 {
      unsafe {
        SelectObject(capture_dc, default_bitmap);
        DeleteObject(HGDIOBJ(capture_bitmap.0));
        DeleteDC(capture_dc);
      }
      return Err(crate::Error::Platform(
        "Failed to create surrogate window.".to_string(),
      ));
    }

    let surrogate = Self {
      hwnd: hwnd.0,
      capture_dc: capture_dc.0,
      capture_bitmap: capture_bitmap.0,
      default_bitmap: default_bitmap.0,
      capture_width: width,
      capture_height: height,
      background_color,
      frozen_rect: source_rect.clone(),
    };

    // Paint the initial frame before the window is made visible.
    surrogate.update(source_rect)?;

    // Place the surrogate immediately above `source_hwnd` in the Z-order
    // and show it without activating it.
    //
    // SAFETY: Both `hwnd` and `source_hwnd` are valid window handles.
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
  /// content at the new dimensions.
  ///
  /// The original snapshot is drawn at its natural size from the top-left
  /// corner of the new frame. Any area not covered is filled with the
  /// sampled background color.
  pub fn update(&self, rect: &Rect) -> crate::Result<()> {
    let new_w = rect.width();
    let new_h = rect.height();

    if new_w <= 0 || new_h <= 0 {
      return Ok(());
    }

    // SAFETY: `GetDC(HWND(0))` returns the screen DC.
    let screen_dc = unsafe { GetDC(HWND(0)) };

    // SAFETY: `screen_dc` is a valid DC.
    let frame_dc = unsafe { CreateCompatibleDC(screen_dc) };
    // SAFETY: `screen_dc` is valid and dimensions are positive.
    let frame_bitmap =
      unsafe { CreateCompatibleBitmap(screen_dc, new_w, new_h) };

    if frame_dc.0 == 0 || frame_bitmap.0 == 0 {
      unsafe { ReleaseDC(HWND(0), screen_dc) };
      if frame_dc.0 != 0 {
        unsafe { DeleteDC(frame_dc) };
      }
      if frame_bitmap.0 != 0 {
        unsafe { DeleteObject(HGDIOBJ(frame_bitmap.0)) };
      }
      return Err(crate::Error::Platform(
        "Failed to create frame DC for surrogate update.".to_string(),
      ));
    }

    // SAFETY: `frame_dc` and `frame_bitmap` are valid GDI objects.
    let old_obj =
      unsafe { SelectObject(frame_dc, HGDIOBJ(frame_bitmap.0)) };

    // Fill the frame with the background color before overlaying the
    // captured content.
    //
    // SAFETY: All parameters are valid GDI objects and structs.
    let brush =
      unsafe { CreateSolidBrush(COLORREF(self.background_color)) };
    let fill = RECT {
      left: 0,
      top: 0,
      right: new_w,
      bottom: new_h,
    };
    unsafe { FillRect(frame_dc, &fill, brush) };
    unsafe { DeleteObject(brush) };

    // Copy the captured snapshot, clipped to the smaller of the capture
    // size and the new frame size.
    let copy_w = self.capture_width.min(new_w);
    let copy_h = self.capture_height.min(new_h);

    if copy_w > 0 && copy_h > 0 {
      // SAFETY: All DC/bitmap objects are valid; dimensions are positive.
      unsafe {
        BitBlt(
          frame_dc,
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

    // `UpdateLayeredWindow` repositions the overlay and updates its content
    // atomically. `ULW_ALPHA` with `SourceConstantAlpha = 255` and
    // `AlphaFormat = 0` renders the window fully opaque without requiring a
    // 32-bit DIB with per-pixel alpha.
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
      cx: new_w,
      cy: new_h,
    };

    // SAFETY: `HWND(self.hwnd)` is a valid layered window; `screen_dc` and
    // all structs are properly initialized.
    let update_result = unsafe {
      UpdateLayeredWindow(
        HWND(self.hwnd),
        screen_dc,
        Some(&raw const pt_dst),
        Some(&raw const sz),
        frame_dc,
        Some(&raw const pt_src),
        COLORREF(0),
        Some(&raw const blend),
        ULW_ALPHA,
      )
    };

    // Clean up the temporary frame DC/bitmap regardless of the update result.
    unsafe {
      ReleaseDC(HWND(0), screen_dc);
      SelectObject(frame_dc, old_obj);
      DeleteObject(HGDIOBJ(frame_bitmap.0));
      DeleteDC(frame_dc);
    }

    update_result?;
    Ok(())
  }

  /// Samples a representative background color from the captured bitmap.
  ///
  /// Reads pixels at a 3×3 grid spread across the surface, filters out
  /// `CLR_INVALID` results, and returns the median value. Falls back to
  /// black if all samples are invalid.
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
      // CLR_INVALID (0xFFFF_FFFF) is returned for out-of-bounds coordinates.
      .filter(|&c| c != 0xFFFF_FFFF)
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
    // SAFETY: All handles were created in `create` and remain valid here.
    // Restoring the default bitmap before deletion prevents GDI leaks.
    unsafe {
      SelectObject(HDC(self.capture_dc), HGDIOBJ(self.default_bitmap));
      DeleteObject(HGDIOBJ(self.capture_bitmap));
      DeleteDC(HDC(self.capture_dc));
      let _ = DestroyWindow(HWND(self.hwnd));
    }
  }
}
