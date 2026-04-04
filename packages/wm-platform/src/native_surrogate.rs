use std::{ffi::c_void, sync::OnceLock};

use windows::{
  core::{s, w},
  Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::{
      Dwm::{
        DwmRegisterThumbnail, DwmUnregisterThumbnail,
        DwmUpdateThumbnailProperties, DWM_THUMBNAIL_PROPERTIES,
        DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION, DWM_TNP_RECTSOURCE,
        DWM_TNP_SOURCECLIENTAREAONLY, DWM_TNP_VISIBLE,
      },
      Gdi::{
        CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
        GetDC, GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER,
        DIB_RGB_COLORS, HGDIOBJ,
      },
    },
    Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS},
    System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW,
      SetWindowPos, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE,
      SWP_NOSENDCHANGING, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, WNDCLASSW,
      WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::{Color, Rect, SurrogateBackdrop};

/// Ensures the surrogate window class is registered exactly once per process.
static SURROGATE_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// Cached pointer to the undocumented `SetWindowCompositionAttribute` export.
static SET_WCA: OnceLock<Option<SetWindowCompositionAttributeFn>> =
  OnceLock::new();

type SetWindowCompositionAttributeFn =
  unsafe extern "system" fn(HWND, *mut WindowCompositionAttribData) -> i32;

/// Accent state value for a solid-color fill.
const ACCENT_ENABLE_GRADIENT: u32 = 1;
/// Accent state value for Windows 10 1803+ Acrylic blur-behind.
const ACCENT_ENABLE_ACRYLICBLURBEHIND: u32 = 4;

/// `WCA_ACCENT_POLICY` attribute index for `SetWindowCompositionAttribute`.
const WCA_ACCENT_POLICY: u32 = 19;

/// `PrintWindow` flag to capture hardware-accelerated window content
/// (DX/OpenGL). Available since Windows 8.1.
const PW_RENDERFULLCONTENT: u32 = 0x2;

/// Width of the pixel strip sampled from each edge when using
/// `SurrogateBackdrop::Auto`.
const EDGE_SAMPLE_PX: i32 = 6;

/// Undocumented accent policy passed to `SetWindowCompositionAttribute`.
#[repr(C)]
struct AccentPolicy {
  accent_state: u32,
  accent_flags: u32,
  /// ABGR color applied over the backdrop.
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
      // Null background brush: DWM renders the backdrop; GDI never touches
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
    let module = unsafe { GetModuleHandleW(w!("user32.dll")).ok()? };

    // SAFETY: `module` is a valid handle. The ASCII string is
    // null-terminated via the `s!` macro.
    let proc = unsafe {
      GetProcAddress(module, s!("SetWindowCompositionAttribute"))
    }?;

    // SAFETY: `proc` is a valid export with the expected calling convention
    // and parameter layout.
    Some(unsafe {
      std::mem::transmute::<
        unsafe extern "system" fn() -> isize,
        SetWindowCompositionAttributeFn,
      >(proc)
    })
  })
}

/// Applies a backdrop effect to `hwnd` via the undocumented
/// `SetWindowCompositionAttribute` API (Windows 10 1607+).
///
/// When `color` is `Some`, a solid-color fill (`ACCENT_ENABLE_GRADIENT`) is
/// applied. When `None`, Windows Acrylic blur-behind is used (requires
/// Windows 10 1803+; degrades gracefully on older versions).
///
/// This is a no-op when the API is unavailable (pre-Windows 10 1607).
fn apply_backdrop(hwnd: HWND, color: Option<&Color>) {
  let Some(set_wca) = get_set_wca() else {
    return;
  };

  let (accent_state, gradient_color) = match color {
    Some(c) => {
      // The undocumented `gradient_color` field uses ABGR byte order:
      // alpha in the high byte, then blue, green, red in the low byte.
      let abgr = (u32::from(c.a) << 24)
        | (u32::from(c.b) << 16)
        | (u32::from(c.g) << 8)
        | u32::from(c.r);
      (ACCENT_ENABLE_GRADIENT, abgr)
    }
    None => (ACCENT_ENABLE_ACRYLICBLURBEHIND, 0x0000_0000),
  };

  let mut policy = AccentPolicy {
    accent_state,
    accent_flags: 0,
    gradient_color,
    animation_id: 0,
  };

  let mut data = WindowCompositionAttribData {
    attrib: WCA_ACCENT_POLICY,
    pv_data: std::ptr::addr_of_mut!(policy) as *mut c_void,
    cb_data: std::mem::size_of::<AccentPolicy>(),
  };

  // SAFETY: `hwnd` is a valid window handle. `data` and `policy` are
  // stack-allocated and remain live for the duration of this call. The
  // struct layout matches the undocumented Win32 ABI for `WCA_ACCENT_POLICY`.
  unsafe { set_wca(hwnd, std::ptr::addr_of_mut!(data)) };
}

/// Captures a snapshot of `hwnd` via `PrintWindow` and returns the average
/// RGBA color of a [`EDGE_SAMPLE_PX`]-pixel-wide strip along all four edges.
///
/// This is called once at animation start when `SurrogateBackdrop::Auto` is
/// configured, so the surrogate backdrop blends naturally with the window
/// content as it extends toward the target rect.
///
/// Returns `None` when `PrintWindow` fails (e.g., for some DX12/Vulkan
/// fullscreen-exclusive apps) — the caller should fall back to Acrylic.
fn sample_edge_color(hwnd: HWND, width: i32, height: i32) -> Option<Color> {
  if width <= 2 * EDGE_SAMPLE_PX || height <= 2 * EDGE_SAMPLE_PX {
    return None;
  }

  // SAFETY: All GDI objects are created, selected, and released in the
  // correct order. Early returns release everything acquired so far.
  unsafe {
    let hdc_wnd = GetDC(hwnd);
    if hdc_wnd.0 == 0 {
      return None;
    }

    let hdc_mem = CreateCompatibleDC(hdc_wnd);
    if hdc_mem.0 == 0 {
      ReleaseDC(hwnd, hdc_wnd);
      return None;
    }

    let hbm = CreateCompatibleBitmap(hdc_wnd, width, height);
    if hbm.0 == 0 {
      DeleteDC(hdc_mem);
      ReleaseDC(hwnd, hdc_wnd);
      return None;
    }

    let prev = SelectObject(hdc_mem, HGDIOBJ(hbm.0));

    // `PW_RENDERFULLCONTENT` captures hardware-accelerated content (DX, GL).
    let ok = PrintWindow(
      hwnd,
      hdc_mem,
      PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT),
    )
    .as_bool();

    let color = if ok {
      let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
          biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
          biWidth: width,
          // Negative height requests a top-down bitmap (row 0 = top of image).
          biHeight: -height,
          biPlanes: 1,
          biBitCount: 32,
          // BI_RGB = 0; using the literal avoids importing the newtype.
          biCompression: 0,
          ..Default::default()
        },
        ..Default::default()
      };

      let mut pixels = vec![0u8; (width * height * 4) as usize];

      let lines = GetDIBits(
        hdc_mem,
        hbm,
        0,
        height as u32,
        Some(pixels.as_mut_ptr().cast()),
        &mut bmi,
        DIB_RGB_COLORS,
      );

      if lines > 0 {
        let stride = (width * 4) as usize;
        let (mut r, mut g, mut b, mut n) = (0u64, 0u64, 0u64, 0u64);
        let strip = EDGE_SAMPLE_PX as usize;
        let w = width as usize;
        let h = height as usize;

        for y in 0..h {
          for x in 0..w {
            if x < strip || x >= w - strip || y < strip || y >= h - strip {
              let i = y * stride + x * 4;
              // GDI 32-bpp bitmaps store pixels as BGRA.
              b += u64::from(pixels[i]);
              g += u64::from(pixels[i + 1]);
              r += u64::from(pixels[i + 2]);
              n += 1;
            }
          }
        }

        (n > 0).then(|| Color {
          r: (r / n) as u8,
          g: (g / n) as u8,
          b: (b / n) as u8,
          a: 255,
        })
      } else {
        None
      }
    } else {
      None
    };

    SelectObject(hdc_mem, prev);
    DeleteObject(HGDIOBJ(hbm.0));
    DeleteDC(hdc_mem);
    ReleaseDC(hwnd, hdc_wnd);

    color
  }
}

/// Registers a DWM thumbnail of `source_hwnd` onto `dest_hwnd`, pinned to
/// `width × height` with no scaling.
///
/// Both `rcSource` and `rcDestination` are set to `{0, 0, width, height}` so
/// DWM captures exactly that region of the source window and renders it 1:1
/// into the destination — no upscaling or downscaling occurs even if the
/// source window is later resized to a different size.
///
/// Returns the opaque thumbnail handle, or `None` if registration fails (e.g.
/// same-window, invalid handle). The caller is responsible for calling
/// [`DwmUnregisterThumbnail`] when done.
fn register_thumbnail(
  dest_hwnd: HWND,
  source_hwnd: HWND,
  width: i32,
  height: i32,
) -> Option<isize> {
  // SAFETY: Both handles are valid top-level windows.
  let thumbnail =
    unsafe { DwmRegisterThumbnail(dest_hwnd, source_hwnd).ok()? };

  let pinned_rect = RECT {
    left: 0,
    top: 0,
    right: width,
    bottom: height,
  };

  let props = DWM_THUMBNAIL_PROPERTIES {
    dwFlags: DWM_TNP_RECTDESTINATION
      | DWM_TNP_RECTSOURCE
      | DWM_TNP_OPACITY
      | DWM_TNP_VISIBLE
      | DWM_TNP_SOURCECLIENTAREAONLY,
    rcDestination: pinned_rect,
    rcSource: pinned_rect,
    opacity: 255,
    fVisible: true.into(),
    fSourceClientAreaOnly: true.into(),
    ..Default::default()
  };

  // SAFETY: `thumbnail` is a valid handle returned by `DwmRegisterThumbnail`.
  if unsafe {
    DwmUpdateThumbnailProperties(thumbnail, &raw const props)
  }
  .is_err()
  {
    // SAFETY: Same handle; unregister on failure.
    unsafe { let _ = DwmUnregisterThumbnail(thumbnail); };
    return None;
  }

  Some(thumbnail)
}

/// Lightweight overlay window used during move/resize animations.
///
/// At animation start the overlay is placed over the real app window at the
/// source rect. A configurable backdrop is applied (Acrylic blur, auto-sampled
/// edge color, or an explicit color) and a DWM thumbnail of the real window is
/// rendered on top — showing the window's original content at its original size
/// via a pinned `rcSource` rect. GlazeWM cloaks the real window while the
/// overlay is active.
///
/// Per-frame cost is one [`SetWindowPos`] call. No GDI allocations occur after
/// creation and the thumbnail properties are never updated after creation.
///
/// When the animation finishes the real window is uncloaked and this surrogate
/// is dropped, which unregisters the thumbnail and destroys the overlay window.
///
/// # Platform-specific
///
/// Only available on Windows. Acrylic requires Windows 10 1803+; on older
/// versions the backdrop degrades gracefully (no blur, thumbnail still shown).
pub struct NativeSurrogate {
  /// Handle to the overlay window.
  hwnd: isize,
  /// DWM thumbnail handle, or `0` if registration failed.
  thumbnail: isize,
}

impl NativeSurrogate {
  /// Creates a surrogate overlay and positions it above `source_hwnd`.
  ///
  /// The overlay is shown without activating it. A DWM thumbnail of
  /// `source_hwnd` is registered to display the window's live content. The
  /// backdrop style is controlled by `backdrop`:
  /// - `Acrylic`: Windows Acrylic blur-behind.
  /// - `Auto`: samples the window's edge pixels and uses the average color.
  /// - `Color`: flat fill with the provided color.
  ///
  /// Returns an error if window creation fails.
  pub fn create(
    source_hwnd: HWND,
    source_rect: &Rect,
    backdrop: &SurrogateBackdrop,
  ) -> crate::Result<Self> {
    ensure_class_registered();

    let src_w = source_rect.width();
    let src_h = source_rect.height();

    // For `Auto`, sample the window's edge pixels before the surrogate covers
    // it. Failure falls back to `None` (Acrylic).
    let auto_color: Option<Color>;
    let color_ref: Option<&Color> = match backdrop {
      SurrogateBackdrop::Acrylic => None,
      SurrogateBackdrop::Color(c) => Some(c),
      SurrogateBackdrop::Auto => {
        auto_color = sample_edge_color(source_hwnd, src_w, src_h);
        if auto_color.is_none() {
          tracing::debug!(
            "Auto edge-color sampling failed; falling back to Acrylic."
          );
        }
        auto_color.as_ref()
      }
    };

    // SAFETY: Class name is the static literal registered above.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
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
      return Err(crate::Error::Platform(
        "Failed to create surrogate window.".to_string(),
      ));
    }

    // Apply the backdrop. When the thumbnail is opaque this is only visible
    // in areas the thumbnail doesn't cover (i.e. when the surrogate grows
    // beyond the source window's dimensions).
    apply_backdrop(hwnd, color_ref);

    // Register a DWM thumbnail of the source window so its live content is
    // rendered inside the surrogate. Failure is non-fatal: the surrogate
    // falls back to backdrop-only.
    let thumbnail =
      register_thumbnail(hwnd, source_hwnd, src_w, src_h).unwrap_or(0);

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
      thumbnail,
    })
  }

  /// Moves and resizes the surrogate overlay to `rect`.
  ///
  /// The DWM thumbnail remains pinned at the original source window size set
  /// during [`create`]; only the backdrop animates. The thumbnail fills the
  /// surrogate exactly when it matches the source size and reveals the
  /// backdrop as the surrogate grows beyond it.
  ///
  /// [`create`]: NativeSurrogate::create
  pub fn update(&mut self, rect: &Rect) -> crate::Result<()> {
    // SAFETY: `HWND(self.hwnd)` is the overlay window created in `create`
    // and remains valid until `drop`. With `SWP_NOZORDER` set,
    // `hWndInsertAfter` is ignored per the Win32 documentation.
    unsafe {
      SetWindowPos(
        HWND(self.hwnd),
        HWND(0),
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOSENDCHANGING | SWP_NOZORDER,
      )
    }?;

    Ok(())
  }
}

impl Drop for NativeSurrogate {
  fn drop(&mut self) {
    // SAFETY: `self.thumbnail` and `self.hwnd` are valid handles created in
    // `create`. Thumbnail must be unregistered before the destination window
    // is destroyed.
    unsafe {
      if self.thumbnail != 0 {
        let _ = DwmUnregisterThumbnail(self.thumbnail);
      }
      let _ = DestroyWindow(HWND(self.hwnd));
    }
  }
}
