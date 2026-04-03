use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{BOOL, HWND, LPARAM, LRESULT, WPARAM},
    Graphics::{
      Dwm::{
        DwmExtendFrameIntoClientArea, DwmSetWindowAttribute,
        DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
        DWMWA_USE_HOSTBACKDROPBRUSH,
      },
      Gdi::{GetStockObject, BLACK_BRUSH, HBRUSH},
    },
    UI::{
      Controls::MARGINS,
      WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW,
        SetWindowPos, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
        SWP_SHOWWINDOW, WNDCLASSW, WM_NCCALCSIZE, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP, WS_THICKFRAME,
      },
    },
  },
};

use crate::Rect;

/// Ensures the surrogate window class is registered exactly once per process.
static SURROGATE_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// Window procedure for the surrogate overlay.
///
/// Handles `WM_NCCALCSIZE` by returning 0, which removes all non-client
/// borders while preserving `WS_THICKFRAME`'s side-effect of enabling
/// DWM system-backdrop rendering on the window.
unsafe extern "system" fn surrogate_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  // Returning 0 for WM_NCCALCSIZE (when wParam is non-zero) removes all
  // non-client chrome. WS_THICKFRAME is still present in the window style,
  // which satisfies DWM's requirement for system-backdrop activation, but
  // no visible border or shadow is rendered.
  if msg == WM_NCCALCSIZE && wparam.0 != 0 {
    return LRESULT(0);
  }

  // SAFETY: All parameters are passed through unchanged.
  unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn ensure_class_registered() {
  SURROGATE_CLASS_REGISTERED.get_or_init(|| {
    // Black background brush: `DwmExtendFrameIntoClientArea` with all-negative
    // margins maps GDI black (0, 0, 0) to transparent, allowing the Acrylic
    // backdrop to fill the window. NULL brush would leave uninitialized pixels
    // that block the effect.
    //
    // SAFETY: `GetStockObject(BLACK_BRUSH)` always succeeds.
    let black_brush =
      HBRUSH(unsafe { GetStockObject(BLACK_BRUSH).0 as isize });

    let wnd_class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_Surrogate"),
      lpfnWndProc: Some(surrogate_wnd_proc),
      hbrBackground: black_brush,
      ..Default::default()
    };

    // SAFETY: `wnd_class` is a properly initialized `WNDCLASSW` with a
    // static class name and a valid window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
}

/// Lightweight Acrylic-blur overlay window used during move/resize animations.
///
/// Uses `DWMSBT_TRANSIENTWINDOW` (Windows 11 22H2+) to render a frosted-glass
/// backdrop via DWM. No GDI rendering or screenshot capture is involved; DWM
/// composites the blur entirely on the GPU.
///
/// The recipe: `WS_POPUP | WS_THICKFRAME` (DWM requires a frame style for
/// backdrop activation), `WM_NCCALCSIZE → 0` (strips the visible border),
/// black background (mapped to transparent by `DwmExtendFrameIntoClientArea`),
/// `DWMWA_USE_HOSTBACKDROPBRUSH = TRUE` (required for Desktop Acrylic on
/// Win32), then `DWMSBT_TRANSIENTWINDOW`.
///
/// Falls back to direct per-frame `SetWindowPos` animation on systems where
/// `DWMSBT_TRANSIENTWINDOW` is not supported (pre-22H2).
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeSurrogate {
  /// Handle to the overlay window.
  hwnd: isize,
}

impl NativeSurrogate {
  /// Creates and displays an Acrylic surrogate at `source_rect`.
  ///
  /// Returns an error if the system does not support `DWMSBT_TRANSIENTWINDOW`
  /// or if window creation fails, so the caller can fall back gracefully.
  pub fn create(
    source_hwnd: HWND,
    source_rect: &Rect,
    _target_rect: &Rect,
  ) -> crate::Result<Self> {
    ensure_class_registered();

    // `WS_THICKFRAME` is required for DWM to activate system-backdrop
    // rendering. The visible resize border it normally adds is stripped
    // by the `WM_NCCALCSIZE` handler above.
    // Neither `WS_EX_LAYERED` nor `WS_EX_NOREDIRECTIONBITMAP` are used:
    // the DWM standard redirection surface must be present for the backdrop.
    let ex_style = WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT;

    // SAFETY: Class name is the static literal registered above.
    let hwnd = unsafe {
      CreateWindowExW(
        ex_style,
        w!("GlazeWM_Surrogate"),
        w!(""),
        WS_POPUP | WS_THICKFRAME,
        source_rect.x(),
        source_rect.y(),
        source_rect.width(),
        source_rect.height(),
        None,
        None,
        None,
        None,
      )
    };

    if hwnd.0 == 0 {
      return Err(crate::Error::Platform(
        "Failed to create surrogate window.".to_string(),
      ));
    }

    // Extend the DWM non-client frame over the entire client area. GDI black
    // in the client area is treated as transparent, so the Acrylic backdrop
    // fills the whole window surface.
    let margins = MARGINS {
      cxLeftWidth: -1,
      cxRightWidth: -1,
      cyTopHeight: -1,
      cyBottomHeight: -1,
    };

    // SAFETY: `hwnd` is a valid top-level window.
    if let Err(err) =
      unsafe { DwmExtendFrameIntoClientArea(hwnd, &margins) }
    {
      unsafe { let _ = DestroyWindow(hwnd); }
      return Err(crate::Error::Platform(format!(
        "DwmExtendFrameIntoClientArea failed: {err}."
      )));
    }

    // Required for Desktop Acrylic (`DWMSBT_TRANSIENTWINDOW`) on Win32.
    // Without this, the backdrop type alone has no effect.
    let use_host_backdrop: BOOL = BOOL(1);

    // SAFETY: pointer and size are correct for a `BOOL`.
    if let Err(err) = unsafe {
      DwmSetWindowAttribute(
        hwnd,
        DWMWA_USE_HOSTBACKDROPBRUSH,
        &use_host_backdrop as *const BOOL as *const std::ffi::c_void,
        std::mem::size_of::<BOOL>() as u32,
      )
    } {
      unsafe { let _ = DestroyWindow(hwnd); }
      return Err(crate::Error::Platform(format!(
        "DWMWA_USE_HOSTBACKDROPBRUSH failed: {err}."
      )));
    }

    // Request Acrylic blur backdrop. Requires Windows 11 22H2 (Build 22621)+.
    let backdrop = DWMSBT_TRANSIENTWINDOW.0;

    // SAFETY: `backdrop` is an `i32`; pointer and size are valid.
    if let Err(err) = unsafe {
      DwmSetWindowAttribute(
        hwnd,
        DWMWA_SYSTEMBACKDROP_TYPE,
        &backdrop as *const i32 as *const std::ffi::c_void,
        std::mem::size_of::<i32>() as u32,
      )
    } {
      unsafe { let _ = DestroyWindow(hwnd); }
      return Err(crate::Error::Platform(format!(
        "DWMSBT_TRANSIENTWINDOW not supported: {err}."
      )));
    }

    // Place the surrogate immediately above `source_hwnd` in Z-order and
    // show it without activating it.
    //
    // SAFETY: Both handles are valid.
    if let Err(err) = unsafe {
      SetWindowPos(
        hwnd,
        source_hwnd,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
      )
    } {
      unsafe { let _ = DestroyWindow(hwnd); }
      return Err(crate::Error::Platform(format!(
        "Failed to show surrogate window: {err}."
      )));
    }

    Ok(Self { hwnd: hwnd.0 })
  }

  /// Moves and resizes the surrogate overlay to `rect`.
  ///
  /// DWM re-renders the Acrylic backdrop at the new geometry on the GPU;
  /// no application-level painting is required.
  pub fn update(&mut self, rect: &Rect) -> crate::Result<()> {
    // SAFETY: `hwnd` is a valid window handle obtained in `create`.
    unsafe {
      SetWindowPos(
        HWND(self.hwnd),
        HWND(0),
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_NOZORDER,
      )
    }?;

    Ok(())
  }
}

impl Drop for NativeSurrogate {
  fn drop(&mut self) {
    // SAFETY: `hwnd` was obtained in `create` and remains valid until drop.
    unsafe { let _ = DestroyWindow(HWND(self.hwnd)); }
  }
}
