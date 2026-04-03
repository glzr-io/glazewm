use std::{ffi::c_void, sync::OnceLock};

use windows::{
  core::{s, w},
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW,
      SetWindowPos, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
      SWP_SHOWWINDOW, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
      WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::Rect;

/// Ensures the surrogate window class is registered exactly once per process.
static SURROGATE_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// Cached pointer to the undocumented `SetWindowCompositionAttribute` export.
static SET_WCA: OnceLock<Option<SetWindowCompositionAttributeFn>> =
  OnceLock::new();

type SetWindowCompositionAttributeFn =
  unsafe extern "system" fn(HWND, *mut WindowCompositionAttribData) -> i32;

/// Accent state value for Windows 10 1803+ Acrylic blur-behind.
const ACCENT_ENABLE_ACRYLICBLURBEHIND: u32 = 4;

/// `WCA_ACCENT_POLICY` attribute index for `SetWindowCompositionAttribute`.
const WCA_ACCENT_POLICY: u32 = 19;

/// Undocumented accent policy passed to `SetWindowCompositionAttribute`.
#[repr(C)]
struct AccentPolicy {
  accent_state: u32,
  accent_flags: u32,
  /// ARGB tint applied over the blurred backdrop.
  gradient_color: u32,
  animation_id: u32,
}

/// Descriptor for `SetWindowCompositionAttribute`.
#[repr(C)]
struct WindowCompositionAttribData {
  attrib: u32,
  pv_data: *mut c_void,
  cb_data: usize,
}

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
      // Null background brush: DWM renders the acrylic; GDI never touches
      // the client area.
      ..Default::default()
    };

    // SAFETY: `wnd_class` is a properly initialized `WNDCLASSW` with a
    // static class name and a valid window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
}

/// Retrieves the `SetWindowCompositionAttribute` function pointer from
/// user32.dll, caching it in [`SET_WCA`] for subsequent calls.
///
/// Returns `None` when the export is unavailable (pre-Windows 10 1607).
fn get_set_wca() -> Option<SetWindowCompositionAttributeFn> {
  *SET_WCA.get_or_init(|| {
    // SAFETY: user32.dll is always loaded in every Win32 process.
    // `GetModuleHandleW` does not increment the reference count.
    let module =
      unsafe { GetModuleHandleW(w!("user32.dll")).ok()? };

    // SAFETY: `module` is a valid handle. The ASCII string is
    // null-terminated via the `s!` macro.
    let proc = unsafe {
      GetProcAddress(module, s!("SetWindowCompositionAttribute"))
    }?;

    // SAFETY: `proc` is a valid export with the expected calling
    // convention and parameter layout.
    Some(unsafe {
      std::mem::transmute::<
        unsafe extern "system" fn() -> isize,
        SetWindowCompositionAttributeFn,
      >(proc)
    })
  })
}

/// Applies Windows Acrylic blur-behind to `hwnd` via the undocumented
/// `SetWindowCompositionAttribute` API (Windows 10 1803+).
///
/// On older Windows versions or when the API is unavailable this is a no-op;
/// the surrogate window will still appear and animate, just without the blur.
fn apply_acrylic(hwnd: HWND) {
  let Some(set_wca) = get_set_wca() else {
    return;
  };

  let mut policy = AccentPolicy {
    accent_state: ACCENT_ENABLE_ACRYLICBLURBEHIND,
    accent_flags: 0,
    // Zero ARGB tint: no color overlay, pure blur effect.
    gradient_color: 0x0000_0000,
    animation_id: 0,
  };

  let mut data = WindowCompositionAttribData {
    attrib: WCA_ACCENT_POLICY,
    pv_data: std::ptr::addr_of_mut!(policy) as *mut c_void,
    cb_data: std::mem::size_of::<AccentPolicy>(),
  };

  // SAFETY: `hwnd` is a valid window handle. `data` and `policy` are
  // stack-allocated and remain live for the duration of the call. The struct
  // layout matches the undocumented Win32 ABI for `WCA_ACCENT_POLICY`.
  unsafe { set_wca(hwnd, std::ptr::addr_of_mut!(data)) };
}

/// Lightweight overlay window used during move/resize animations.
///
/// At animation start the overlay is placed over the real app window at the
/// source rect with Windows Acrylic blur-behind applied. Each frame the
/// overlay is moved/resized via `SetWindowPos` — no GDI allocations at any
/// point. GlazeWM cloaks the real window while the overlay is active so only
/// the surrogate is visible. When the animation finishes the real window is
/// repositioned, uncloaked, and this surrogate is dropped.
///
/// # Platform-specific
///
/// Only available on Windows. Requires Windows 10 1803+ for Acrylic; on
/// older versions the backdrop degrades gracefully (no blur).
pub struct NativeSurrogate {
  /// Handle to the overlay window.
  hwnd: isize,

  /// Position of the real app window at the moment the animation started.
  pub frozen_rect: Rect,
}

impl NativeSurrogate {
  /// Creates an acrylic surrogate overlay and positions it above
  /// `source_hwnd`.
  ///
  /// The overlay is shown without activating it and without taking focus.
  /// Returns an error if window creation fails.
  pub fn create(
    source_hwnd: HWND,
    source_rect: &Rect,
    _target_rect: &Rect,
  ) -> crate::Result<Self> {
    ensure_class_registered();

    // SAFETY: Class name is the static literal registered above.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
        w!("GlazeWM_Surrogate"),
        w!(""),
        WS_POPUP,
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

    apply_acrylic(hwnd);

    // Place the surrogate immediately above `source_hwnd` in the Z-order
    // and show it without activating it.
    //
    // SAFETY: Both handles are valid.
    unsafe {
      SetWindowPos(
        hwnd,
        source_hwnd,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
      )
    }?;

    Ok(Self {
      hwnd: hwnd.0,
      frozen_rect: source_rect.clone(),
    })
  }

  /// Moves and resizes the surrogate overlay to `rect`.
  ///
  /// Only calls `SetWindowPos` — no GDI or DWM attribute updates are needed
  /// per frame since DWM redraws the acrylic backdrop automatically.
  pub fn update(&mut self, rect: &Rect) -> crate::Result<()> {
    // SAFETY: `HWND(self.hwnd)` is the overlay window created in `create`
    // and remains valid until `drop`. With `SWP_NOZORDER` set,
    // `hWndInsertAfter` (`HWND(0)`) is ignored per the Win32 documentation.
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
    // SAFETY: `HWND(self.hwnd)` is valid and was created in `create`.
    unsafe {
      let _ = DestroyWindow(HWND(self.hwnd));
    }
  }
}
