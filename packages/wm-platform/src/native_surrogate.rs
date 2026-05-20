use std::{ffi::c_void, sync::OnceLock};

use windows::{
  core::{s, w},
  Win32::{
    Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Dwm::{
      DwmExtendFrameIntoClientArea, DwmRegisterThumbnail,
      DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
      DWM_THUMBNAIL_PROPERTIES, DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION,
      DWM_TNP_RECTSOURCE, DWM_TNP_SOURCECLIENTAREAONLY, DWM_TNP_VISIBLE,
    },
    System::LibraryLoader::{GetModuleHandleW, GetProcAddress},
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW,
      SetLayeredWindowAttributes, SetWindowPos, SET_WINDOW_POS_FLAGS,
      LWA_ALPHA, SWP_NOACTIVATE, SWP_NOCOPYBITS, SWP_NOMOVE,
      SWP_NOSENDCHANGING, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, WNDCLASSW,
      WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
      WS_POPUP,
    },
  },
};

use crate::{Color, Rect};

/// Ensures the surrogate window class is registered exactly once per
/// process.
static SURROGATE_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// Cached pointer to the undocumented `SetWindowCompositionAttribute`
/// export.
static SET_WCA: OnceLock<Option<SetWindowCompositionAttributeFn>> =
  OnceLock::new();

type SetWindowCompositionAttributeFn =
  unsafe extern "system" fn(HWND, *mut WindowCompositionAttribData) -> i32;

/// Accent state value for a solid-color fill.
const ACCENT_ENABLE_GRADIENT: u32 = 1;

/// `WCA_ACCENT_POLICY` attribute index for
/// `SetWindowCompositionAttribute`.
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

/// Default window procedure wrapper with the required `extern "system"`
/// ABI.
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
      // Null background brush: DWM renders acrylic; GDI never touches the
      // client area.
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

/// Applies a solid-color backdrop to `hwnd` via the undocumented
/// `SetWindowCompositionAttribute` API (Windows 10 1607+).
///
/// When `color` is `None`, no accent is applied — DWM's default transparent
/// backing store is used so the border-extension area around the DWM thumbnail
/// is genuinely see-through.
///
/// This is a no-op when the API is unavailable (pre-Windows 10 1607).
fn apply_backdrop(hwnd: HWND, color: Option<&Color>) {
  let Some(c) = color else {
    return;
  };

  // The undocumented `gradient_color` field uses ABGR byte order:
  // alpha in the high byte, then blue, green, red in the low bytes.
  let abgr = (u32::from(c.a) << 24)
    | (u32::from(c.b) << 16)
    | (u32::from(c.g) << 8)
    | u32::from(c.r);

  let (accent_state, gradient_color) = (ACCENT_ENABLE_GRADIENT, abgr);

  let Some(set_wca) = get_set_wca() else {
    return;
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
  // struct layout matches the undocumented Win32 ABI for
  // `WCA_ACCENT_POLICY`.
  unsafe { set_wca(hwnd, std::ptr::addr_of_mut!(data)) };
}

/// Registers a DWM thumbnail of `source_hwnd` onto `dest_hwnd`.
///
/// `logical_width` and `logical_height` are the visible content dimensions
/// of the source window (physical size minus invisible border). `border_inset`
/// gives the per-side border widths in the source window's coordinate space.
///
/// `rcSource` is set to the visible content area of the source window
/// (offset by `border_inset`). `rcDestination` fills the surrogate at
/// `{0, 0, logical_width, logical_height}` — callers are expected to have
/// already sized the surrogate to the logical rect. When `border_inset` is
/// all-zero the behaviour is identical to passing the full physical dimensions.
///
/// Returns the opaque thumbnail handle, or `None` if registration fails
/// (e.g. same-window, invalid handle). The caller is responsible for
/// calling [`DwmUnregisterThumbnail`] when done.
fn register_thumbnail(
  dest_hwnd: HWND,
  source_hwnd: HWND,
  logical_width: i32,
  logical_height: i32,
  border_inset: RECT,
) -> Option<isize> {
  // SAFETY: Both handles are valid top-level windows.
  let thumbnail =
    unsafe { DwmRegisterThumbnail(dest_hwnd, source_hwnd).ok()? };

  // `rcSource` starts at the border inset so invisible-border pixels are
  // excluded; those pixels render as black in DWM thumbnails. `rcDestination`
  // fills the whole (logical-sized) surrogate from (0, 0).
  let src_rect = RECT {
    left: border_inset.left,
    top: border_inset.top,
    right: border_inset.left + logical_width,
    bottom: border_inset.top + logical_height,
  };
  let dst_rect = RECT {
    left: 0,
    top: 0,
    right: logical_width,
    bottom: logical_height,
  };

  let props = DWM_THUMBNAIL_PROPERTIES {
    dwFlags: DWM_TNP_RECTDESTINATION
      | DWM_TNP_RECTSOURCE
      | DWM_TNP_OPACITY
      | DWM_TNP_VISIBLE
      | DWM_TNP_SOURCECLIENTAREAONLY,
    rcDestination: dst_rect,
    rcSource: src_rect,
    opacity: 255,
    fVisible: true.into(),
    fSourceClientAreaOnly: false.into(),
    ..Default::default()
  };

  // SAFETY: `thumbnail` is a valid handle returned by
  // `DwmRegisterThumbnail`.
  if unsafe { DwmUpdateThumbnailProperties(thumbnail, &raw const props) }
    .is_err()
  {
    // SAFETY: Same handle; unregister on failure.
    unsafe {
      let _ = DwmUnregisterThumbnail(thumbnail);
    };
    return None;
  }

  Some(thumbnail)
}

/// Converts a physical `Rect` to logical by subtracting the invisible border
/// inset on each side.
pub(crate) fn to_logical(rect: &Rect, inset: &RECT) -> Rect {
  Rect::from_ltrb(
    rect.left + inset.left,
    rect.top + inset.top,
    rect.right - inset.right,
    rect.bottom - inset.bottom,
  )
}

/// Lightweight overlay window used during move/resize animations.
///
/// At animation start the overlay is placed over the real app window at the
/// source rect. A DWM thumbnail of the real window is rendered on top. For
/// shrinking animations the thumbnail is registered at the source dimensions
/// so it fills the surrogate initially; as the surrogate shrinks the
/// thumbnail edge is clipped — a wipe effect with no distortion. For growing
/// animations the thumbnail is registered at the target dimensions; as the
/// surrogate expands it progressively reveals the real window's content —
/// a curtain-reveal effect.
///
/// GlazeWM cloaks the real window while the overlay is active.
///
/// Per-frame cost is one [`SetWindowPos`] call (plus one
/// `DwmUpdateThumbnailProperties` when the thumbnail handle is valid). No
/// GDI allocations occur.
///
/// When the animation finishes the real window is uncloaked and this
/// surrogate is dropped, which unregisters the thumbnail and destroys the
/// overlay window.
///
/// # Platform-specific
///
/// Only available on Windows. Acrylic requires Windows 10 1803+; on older
/// versions the backdrop degrades gracefully (no blur, thumbnail still
/// shown).
pub struct NativeSurrogate {
  /// Handle to the overlay window.
  hwnd: isize,
  /// DWM thumbnail handle, or `0` if registration failed.
  thumbnail: isize,
  /// Cached visibility state; guards against redundant `ShowWindow` calls.
  is_visible: bool,
  /// Last opacity passed to `SetLayeredWindowAttributes`; used to skip
  /// redundant calls when opacity has not changed between frames.
  last_opacity: u8,
  /// Last rect passed to `SetWindowPos` via `reposition`; used to skip
  /// redundant calls when the position and size have not changed.
  last_rect: Option<Rect>,
}

impl NativeSurrogate {
  /// Creates a surrogate overlay and positions it above `source_hwnd`.
  ///
  /// The overlay is shown without activating it. A DWM thumbnail of
  /// `source_hwnd` is registered and the surrogate window starts at
  /// `source_rect`. When `surrogate_color` is `Some`, the backdrop is a
  /// solid-color fill; when `None`, the backdrop is fully transparent so only
  /// the DWM thumbnail is visible.
  ///
  /// `thumbnail_rect` controls the DWM thumbnail registration size:
  /// - Pass `source_rect` for shrinking animations — the thumbnail fills the
  ///   whole surrogate at start and the surrogate clips the edge as it shrinks
  ///   (wipe effect, no timing dependency on the real window re-rendering).
  /// - Pass the target rect for growing animations — the thumbnail is
  ///   registered at the final dimensions so the surrogate progressively
  ///   reveals the real window's content as it expands (curtain-reveal).
  ///   The caller must synchronously pre-position the cloaked real window at
  ///   the target rect before animation begins so DWM captures the correctly-
  ///   sized content.
  ///
  /// When `initially_visible` is `false`, the surrogate window is created
  /// hidden; the caller must call [`set_visible`] to reveal it. Pass
  /// `true` for surrogate types that must appear immediately (e.g.
  /// resize sessions). Workspace-switch surrogates pass `false` to avoid
  /// a one-frame flash before the caller explicitly shows the window.
  ///
  /// `border_inset` shrinks the surrogate from the physical rect to the
  /// logical (visible-content) rect, preventing the surrogate from occupying
  /// the configured window gap. Pass `RECT::default()` to keep the full
  /// physical size (workspace-switch surrogates).
  ///
  /// Returns an error if window creation fails.
  ///
  /// [`set_visible`]: NativeSurrogate::set_visible
  pub fn create(
    source_hwnd: HWND,
    source_rect: &Rect,
    thumbnail_rect: &Rect,
    surrogate_color: Option<&Color>,
    opacity: u8,
    initially_visible: bool,
    border_inset: RECT,
  ) -> crate::Result<Self> {
    ensure_class_registered();

    // Surrogate window is sized to the logical source rect (does not occupy
    // the window gap). Thumbnail dimensions come from `thumbnail_rect` and
    // may differ (e.g. target rect for growing animations).
    let logical_src = to_logical(source_rect, &border_inset);
    let logical_thumb = to_logical(thumbnail_rect, &border_inset);

    // SAFETY: Class name is the static literal registered above.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
        w!("GlazeWM_Surrogate"),
        w!(""),
        WS_POPUP,
        logical_src.x(),
        logical_src.y(),
        logical_src.width(),
        logical_src.height(),
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

    // Extend the DWM glass sheet over the entire client area so that regions
    // not covered by the DWM thumbnail are transparent rather than opaque
    // black (which is the GDI default for a `WS_POPUP` with a null background
    // brush). The thumbnail is composited on top of this transparent sheet, so
    // only the thumbnail area shows content; everything else is see-through.
    {
      use windows::Win32::UI::Controls::MARGINS;
      let margins = MARGINS {
        cxLeftWidth: -1,
        cxRightWidth: -1,
        cyTopHeight: -1,
        cyBottomHeight: -1,
      };
      // SAFETY: `hwnd` is a valid window handle. `margins` is stack-allocated
      // and live for the duration of this call.
      unsafe {
        let _ = DwmExtendFrameIntoClientArea(hwnd, &raw const margins);
      }
    }

    apply_backdrop(hwnd, surrogate_color);

    // Set the initial whole-window opacity. `LWA_ALPHA` makes `crKey`
    // irrelevant; COLORREF(0) is a placeholder.
    //
    // SAFETY: `hwnd` is a valid window handle created above.
    unsafe {
      let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), opacity, LWA_ALPHA);
    }

    // Register the DWM thumbnail at `thumbnail_rect` dimensions. For shrinking
    // animations this equals `source_rect` so the thumbnail fills the whole
    // surrogate at start (wipe/clip effect). For growing animations this equals
    // the target rect so the surrogate progressively reveals the real window's
    // final content as it expands (curtain-reveal).
    //
    // Failure is non-fatal: the surrogate still shows its backdrop color if
    // configured.
    let thumbnail = register_thumbnail(
      hwnd,
      source_hwnd,
      logical_thumb.width(),
      logical_thumb.height(),
      border_inset,
    )
    .unwrap_or(0);

    // Place the surrogate immediately above `source_hwnd` in the Z-order.
    // Visibility is controlled by `initially_visible`: workspace-switch
    // surrogates start hidden to avoid a one-frame flash; resize-session
    // surrogates start visible so they cover the real window immediately.
    //
    // SAFETY: Both handles are valid.
    let show_flag = if initially_visible {
      SWP_SHOWWINDOW
    } else {
      SET_WINDOW_POS_FLAGS::default()
    };
    unsafe {
      SetWindowPos(
        hwnd,
        source_hwnd,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | show_flag,
      )
    }?;

    Ok(Self {
      hwnd: hwnd.0,
      thumbnail,
      is_visible: initially_visible,
      last_opacity: opacity,
      last_rect: None,
    })
  }

  /// Returns the raw handle of the surrogate overlay window.
  pub fn hwnd(&self) -> HWND {
    HWND(self.hwnd)
  }

  /// Returns `true` when a DWM thumbnail was successfully registered.
  ///
  /// `DwmRegisterThumbnail` fails for elevated or UWP source windows.
  /// Callers use this to decide whether to freeze the real window behind
  /// the surrogate or fall back to direct repositioning.
  pub fn has_thumbnail(&self) -> bool {
    self.thumbnail != 0
  }

  /// Shows or hides the surrogate overlay window without activating it.
  ///
  /// No-op when the window is already in the requested state.
  pub fn set_visible(&mut self, visible: bool) {
    if self.is_visible == visible {
      return;
    }
    self.is_visible = visible;
    use windows::Win32::UI::WindowsAndMessaging::{
      ShowWindow, SW_HIDE, SW_SHOWNOACTIVATE,
    };
    // SAFETY: `HWND(self.hwnd)` is valid until `drop`.
    unsafe {
      ShowWindow(
        HWND(self.hwnd),
        if visible { SW_SHOWNOACTIVATE } else { SW_HIDE },
      );
    }
  }

  /// Repositions the surrogate overlay to `rect` without touching the DWM
  /// thumbnail properties.
  ///
  /// Use this when the thumbnail is managed separately (e.g. workspace-switch
  /// slide animations that update `rcSource`/`rcDestination` independently).
  /// No-op when `rect` matches the last applied position.
  pub fn reposition(&mut self, rect: &Rect) -> crate::Result<()> {
    if self.last_rect.as_ref() == Some(rect) {
      return Ok(());
    }
    // SAFETY: `HWND(self.hwnd)` is the overlay created in `create` and remains
    // valid until `drop`. `SWP_NOZORDER` makes `hWndInsertAfter` irrelevant.
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
    self.last_rect = Some(rect.clone());
    Ok(())
  }

  /// Sets the DWM thumbnail visibility flag without changing any other
  /// thumbnail properties.
  ///
  /// No-op when no thumbnail was registered.
  pub fn set_thumbnail_visible(&self, visible: bool) {
    if self.thumbnail == 0 {
      return;
    }
    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_VISIBLE,
      fVisible: visible.into(),
      ..Default::default()
    };
    // SAFETY: `self.thumbnail` is a valid handle. `props` is stack-allocated.
    unsafe {
      let _ = DwmUpdateThumbnailProperties(self.thumbnail, &raw const props);
    }
  }

  /// Sets the whole-window opacity via `SetLayeredWindowAttributes`.
  ///
  /// `opacity` ranges from 0 (fully transparent) to 255 (fully opaque).
  /// Composited by DWM at the window level so both the backdrop and the DWM
  /// thumbnail fade together uniformly. No-op when `opacity` matches the last
  /// applied value, avoiding redundant DWM surface-dirty calls on high-refresh
  /// displays where constant-opacity animations tick at the monitor's frame rate.
  pub fn set_window_opacity(&mut self, opacity: u8) {
    if opacity == self.last_opacity {
      return;
    }
    self.last_opacity = opacity;
    // SAFETY: `HWND(self.hwnd)` is valid until `drop`. `LWA_ALPHA` makes
    // `crKey` irrelevant.
    unsafe {
      let _ = SetLayeredWindowAttributes(
        HWND(self.hwnd),
        COLORREF(0),
        opacity,
        LWA_ALPHA,
      );
    }
  }

  /// Updates the DWM thumbnail source and destination rects in a single call.
  ///
  /// `rc_src` is the source-window-local rect to sample from; `rc_dst` is the
  /// surrogate-local rect to render into. Always forces `fVisible = true`,
  /// `opacity = 255`, and `fSourceClientAreaOnly = false`. Overall opacity is
  /// controlled at the window level via [`set_window_opacity`]. No-op when no
  /// thumbnail was registered.
  ///
  /// [`set_window_opacity`]: NativeSurrogate::set_window_opacity
  pub fn set_thumbnail_rects(&self, rc_src: RECT, rc_dst: RECT) {
    if self.thumbnail == 0 {
      return;
    }
    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_RECTSOURCE
        | DWM_TNP_RECTDESTINATION
        | DWM_TNP_OPACITY
        | DWM_TNP_VISIBLE
        | DWM_TNP_SOURCECLIENTAREAONLY,
      rcSource: rc_src,
      rcDestination: rc_dst,
      opacity: u8::MAX,
      fVisible: true.into(),
      fSourceClientAreaOnly: false.into(),
      ..Default::default()
    };
    // SAFETY: `self.thumbnail` is a valid handle. `props` is stack-allocated.
    unsafe {
      let _ = DwmUpdateThumbnailProperties(self.thumbnail, &raw const props);
    }
  }

  /// Unregisters the current DWM thumbnail and registers a new one at
  /// `logical_width` × `logical_height`.
  ///
  /// Called when a growing animation is redirected to a larger target via
  /// `update_target` so the curtain-reveal correctly covers the newly expanded
  /// area. No-op when no thumbnail was registered.
  pub fn reregister_thumbnail(
    &mut self,
    source_hwnd: HWND,
    logical_width: i32,
    logical_height: i32,
    border_inset: RECT,
  ) {
    // SAFETY: `self.thumbnail` is a valid handle (or 0). Unregistering before
    // re-registering prevents a duplicate thumbnail on the same destination.
    if self.thumbnail != 0 {
      unsafe {
        let _ = DwmUnregisterThumbnail(self.thumbnail);
      }
      self.thumbnail = 0;
    }
    self.thumbnail =
      register_thumbnail(HWND(self.hwnd), source_hwnd, logical_width, logical_height, border_inset)
        .unwrap_or(0);
  }

  /// Moves and resizes the surrogate overlay to `rect` and sets the whole-window
  /// opacity to `opacity` (0 = fully transparent, 255 = opaque).
  pub fn update(&mut self, rect: &Rect, opacity: u8) -> crate::Result<()> {
    self.reposition(rect)?;
    self.set_window_opacity(opacity);
    Ok(())
  }
}

impl Drop for NativeSurrogate {
  fn drop(&mut self) {
    // SAFETY: `self.thumbnail` and `self.hwnd` are valid handles created
    // in `create`. Thumbnail must be unregistered before the
    // destination window is destroyed.
    unsafe {
      if self.thumbnail != 0 {
        let _ = DwmUnregisterThumbnail(self.thumbnail);
      }
      let _ = DestroyWindow(HWND(self.hwnd));
    }
  }
}
