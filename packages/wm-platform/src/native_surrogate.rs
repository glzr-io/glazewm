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
        SetWindowPos, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
        SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, WNDCLASSW, WM_NCCALCSIZE,
        WM_NCHITTEST, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
        WS_THICKFRAME,
      },
    },
  },
};

use crate::Rect;

/// Ensures the surrogate window class is registered exactly once per process.
static SURROGATE_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// Window procedure for the surrogate overlay.
///
/// - `WM_NCCALCSIZE` → 0: removes visible border while keeping `WS_THICKFRAME`
///   present for DWM backdrop activation.
/// - `WM_NCHITTEST` → `HTTRANSPARENT`: mouse events pass through to the window
///   below (replaces `WS_EX_TRANSPARENT`, which blocks DWM compositing).
unsafe extern "system" fn surrogate_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match msg {
    WM_NCCALCSIZE if wparam.0 != 0 => LRESULT(0),
    WM_NCHITTEST => LRESULT(-1), // HTTRANSPARENT — pass mouse through
    _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
  }
}

fn ensure_class_registered() {
  SURROGATE_CLASS_REGISTERED.get_or_init(|| {
    // Black background: DwmExtendFrameIntoClientArea maps GDI (0,0,0) to
    // transparent. NULL brush leaves uninitialized pixels that block the
    // backdrop from rendering.
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

    // SAFETY: `wnd_class` is properly initialized with a static class name.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
}

/// Lightweight Acrylic-blur overlay window used during move/resize animations.
///
/// Uses `DWMSBT_TRANSIENTWINDOW` (Windows 11 22H2+) to render a frosted-glass
/// backdrop via DWM entirely on the GPU. Falls back gracefully on older systems.
///
/// Key recipe:
/// - `WS_POPUP | WS_THICKFRAME` — DWM requires a frame style for backdrop.
/// - No `WS_EX_TRANSPARENT` — that flag blocks DWM compositing; click-through
///   is achieved via `WM_NCHITTEST → HTTRANSPARENT` in the window procedure.
/// - `WM_NCCALCSIZE → 0` — strips the visible resize border.
/// - Black background brush — mapped to transparent by `DwmExtendFrameIntoClientArea`.
/// - Window shown **before** DWM attributes are set, then `SWP_FRAMECHANGED`
///   forces a non-client recalculation that triggers backdrop attachment.
/// - `DWMWA_USE_HOSTBACKDROPBRUSH = TRUE` then `DWMSBT_TRANSIENTWINDOW`.
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
  /// or window creation fails, allowing the caller to fall back to direct
  /// per-frame animation.
  pub fn create(
    source_hwnd: HWND,
    source_rect: &Rect,
    _target_rect: &Rect,
  ) -> crate::Result<Self> {
    ensure_class_registered();

    // No WS_EX_TRANSPARENT — that flag prevents DWM from compositing the
    // backdrop. Click-through is handled by WM_NCHITTEST in the wnd proc.
    let ex_style = WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW;

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

    // Show the window first — DWM may require the window to be visible
    // before backdrop attributes take effect.
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

    // Extend DWM non-client frame over the entire client area. GDI black
    // is treated as transparent so the Acrylic fills the whole surface.
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

    // Required for Desktop Acrylic on plain Win32 windows.
    let use_host_backdrop: BOOL = BOOL(1);

    // SAFETY: pointer and size match `BOOL`.
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

    // Request Acrylic blur backdrop (Windows 11 22H2+).
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

    // Force a non-client area recalculation. This triggers WM_NCCALCSIZE
    // (returning 0 removes the frame) and prompts DWM to attach the
    // backdrop to the window.
    //
    // SAFETY: `hwnd` is valid.
    let _ = unsafe {
      SetWindowPos(
        hwnd,
        HWND(0),
        0,
        0,
        0,
        0,
        SWP_NOMOVE
          | SWP_NOSIZE
          | SWP_NOZORDER
          | SWP_NOACTIVATE
          | SWP_FRAMECHANGED,
      )
    };

    Ok(Self { hwnd: hwnd.0 })
  }

  /// Moves and resizes the surrogate overlay to `rect`.
  ///
  /// DWM re-renders the Acrylic backdrop at the new geometry on the GPU.
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
