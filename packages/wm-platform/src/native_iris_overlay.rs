use std::sync::OnceLock;

use windows::{
  core::w,
  Win32::{
    Foundation::{BOOL, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{
      BeginPaint, BitBlt, CombineRgn, CreateCompatibleBitmap,
      CreateCompatibleDC, CreateEllipticRgn, CreateRectRgn, DeleteDC,
      DeleteObject, EndPaint, GetDC, ReleaseDC, SelectObject, SetWindowRgn,
      UpdateWindow, HBITMAP, HGDIOBJ, HRGN, PAINTSTRUCT, RGN_DIFF, SRCCOPY,
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DefWindowProcW, DestroyWindow, GetClientRect,
      GetWindowLongPtrW, RegisterClassW, SetWindowLongPtrW, SetWindowPos,
      GWLP_USERDATA, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
      SWP_SHOWWINDOW, WM_ERASEBKGND, WM_PAINT, WNDCLASSW, WS_EX_NOACTIVATE,
      WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::Rect;

/// Ensures the iris-overlay window class is registered exactly once per
/// process.
static IRIS_CLASS_REGISTERED: OnceLock<()> = OnceLock::new();

/// Window procedure for the iris overlay.
///
/// Handles `WM_PAINT` by blitting the snapshot bitmap (stashed in
/// `GWLP_USERDATA`) to the window, and suppresses the default background erase
/// to avoid a flash. All other messages fall through to `DefWindowProcW`.
unsafe extern "system" fn iris_wnd_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  match msg {
    // The whole client area is repainted from the snapshot, so skip the
    // default erase to avoid a one-frame flash of the background.
    WM_ERASEBKGND => LRESULT(1),
    WM_PAINT => {
      let mut ps = PAINTSTRUCT::default();
      // SAFETY: `hwnd` is valid; `ps` is a stack out-parameter.
      let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
      // SAFETY: the snapshot `HBITMAP` was stashed in `GWLP_USERDATA` at
      // creation and remains valid until the window is destroyed.
      let raw = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) };
      if hdc.0 != 0 && raw != 0 {
        let bitmap = HBITMAP(raw);
        let mut rc = RECT::default();
        // SAFETY: `hwnd` is valid; `rc` is a stack out-parameter.
        let _ = unsafe { GetClientRect(hwnd, &mut rc) };
        // SAFETY: `hdc` is the valid paint DC from `BeginPaint`.
        let mem_dc = unsafe { CreateCompatibleDC(hdc) };
        if mem_dc.0 != 0 {
          // SAFETY: `mem_dc` is valid; `bitmap` is the snapshot.
          let old = unsafe { SelectObject(mem_dc, HGDIOBJ(bitmap.0)) };
          // SAFETY: both DCs are valid; the snapshot matches the client size.
          let _ = unsafe {
            BitBlt(hdc, 0, 0, rc.right, rc.bottom, mem_dc, 0, 0, SRCCOPY)
          };
          // SAFETY: restore and free the memory DC.
          unsafe {
            SelectObject(mem_dc, old);
            let _ = DeleteDC(mem_dc);
          }
        }
      }
      // SAFETY: `ps` was populated by `BeginPaint`.
      unsafe {
        let _ = EndPaint(hwnd, &ps);
      }
      LRESULT(0)
    }
    // SAFETY: all parameters are passed through unchanged.
    _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
  }
}

/// Registers the iris-overlay window class on first use.
fn ensure_class_registered() {
  IRIS_CLASS_REGISTERED.get_or_init(|| {
    let wnd_class = WNDCLASSW {
      lpszClassName: w!("GlazeWM_Iris"),
      lpfnWndProc: Some(iris_wnd_proc),
      ..Default::default()
    };
    // SAFETY: `wnd_class` is initialized with a static class name and a valid
    // window procedure.
    unsafe { RegisterClassW(&raw const wnd_class) };
  });
}

/// Captures the screen content of `monitor` into a new screen-compatible
/// bitmap.
///
/// Returns the captured `HBITMAP` (owned by the caller — free with
/// `DeleteObject`), or an error if any GDI step fails.
fn capture_monitor(monitor: &Rect) -> crate::Result<HBITMAP> {
  let w = monitor.width();
  let h = monitor.height();
  // SAFETY: `HWND(0)` requests a DC for the entire virtual screen.
  let screen_dc = unsafe { GetDC(HWND(0)) };
  if screen_dc.0 == 0 {
    return Err(crate::Error::Platform(
      "Failed to get screen DC for iris snapshot.".to_string(),
    ));
  }

  // Compute the result without early-returning so the single `ReleaseDC` below
  // always runs. SAFETY: `screen_dc` is valid; every GDI object created here is
  // either freed here or handed back to the caller.
  let result = unsafe {
    let mem_dc = CreateCompatibleDC(screen_dc);
    if mem_dc.0 == 0 {
      Err("Failed to create memory DC for iris snapshot.")
    } else {
      let bitmap = CreateCompatibleBitmap(screen_dc, w, h);
      let out = if bitmap.0 == 0 {
        Err("Failed to create bitmap for iris snapshot.")
      } else {
        let old = SelectObject(mem_dc, HGDIOBJ(bitmap.0));
        // Copy the monitor's region from the screen into the bitmap;
        // `monitor` coordinates share the screen DC's virtual-screen space.
        let blit = BitBlt(
          mem_dc, 0, 0, w, h, screen_dc, monitor.x(), monitor.y(), SRCCOPY,
        );
        SelectObject(mem_dc, old);
        if blit.is_err() {
          let _ = DeleteObject(HGDIOBJ(bitmap.0));
          Err("Failed to capture monitor for iris snapshot.")
        } else {
          Ok(bitmap)
        }
      };
      let _ = DeleteDC(mem_dc);
      out
    }
  };

  // SAFETY: `screen_dc` was obtained from `GetDC(HWND(0))`.
  unsafe {
    ReleaseDC(HWND(0), screen_dc);
  }

  result.map_err(|m| crate::Error::Platform(m.to_string()))
}

/// Topmost snapshot overlay that drives the iris-wipe workspace transition.
///
/// On creation it captures the monitor's current content (the outgoing
/// workspace) into a frozen bitmap shown on a topmost, click-through window
/// fixed at the monitor rect. The caller then switches the real windows
/// underneath; each frame [`set_hole`](Self::set_hole) carves a growing
/// circular hole into the overlay so the live incoming workspace shows
/// through. Dropping the overlay tears down the window and frees the snapshot.
///
/// The hole edge is hard (region-based); a soft/feathered edge would require a
/// per-pixel composite pass and is intentionally out of scope here.
///
/// # Platform-specific
///
/// Windows only.
pub struct NativeIrisOverlay {
  /// Handle to the overlay window.
  hwnd: isize,
  /// Monitor rect; the overlay is fixed here and hole coordinates are mapped
  /// into its local space.
  monitor: Rect,
  /// Snapshot bitmap shown by the overlay; freed on drop.
  bitmap: isize,
}

impl NativeIrisOverlay {
  /// Creates and shows the overlay covering `monitor`, frozen on a snapshot of
  /// whatever is currently composited there.
  ///
  /// The overlay starts fully covering (no hole). Call
  /// [`set_hole`](Self::set_hole) each frame to reveal the content beneath.
  ///
  /// Returns an error if the monitor cannot be captured or the window cannot
  /// be created; callers fall back to an instant (non-animated) switch.
  pub fn create(monitor: &Rect) -> crate::Result<Self> {
    ensure_class_registered();

    let bitmap = capture_monitor(monitor)?;

    // SAFETY: the class is registered above with a static class name.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_TOOLWINDOW
          | WS_EX_NOACTIVATE
          | WS_EX_TRANSPARENT
          | WS_EX_TOPMOST,
        w!("GlazeWM_Iris"),
        w!(""),
        WS_POPUP,
        monitor.x(),
        monitor.y(),
        monitor.width(),
        monitor.height(),
        None,
        None,
        None,
        None,
      )
    };

    if hwnd.0 == 0 {
      // SAFETY: `bitmap` is a valid GDI object that never got attached.
      unsafe {
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
      }
      return Err(crate::Error::Platform(
        "Failed to create iris overlay window.".to_string(),
      ));
    }

    // Stash the snapshot so the window procedure can blit it on `WM_PAINT`.
    // SAFETY: `hwnd` is valid; the stored value is an opaque handle.
    unsafe {
      SetWindowLongPtrW(hwnd, GWLP_USERDATA, bitmap.0);
    }

    // Show the overlay topmost without activating it, then force an immediate
    // paint so the snapshot is on screen before the real windows switch
    // underneath (no blank frame).
    // SAFETY: `hwnd` is the window created above.
    if let Err(err) = unsafe {
      SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
      )
    } {
      // SAFETY: clean up the window and snapshot on failure.
      unsafe {
        let _ = DestroyWindow(hwnd);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
      }
      return Err(crate::Error::Platform(format!(
        "Failed to show iris overlay: {err}."
      )));
    }

    // SAFETY: `hwnd` is valid; forces a synchronous `WM_PAINT`.
    unsafe {
      let _ = UpdateWindow(hwnd);
    }

    Ok(Self {
      hwnd: hwnd.0,
      monitor: monitor.clone(),
      bitmap: bitmap.0,
    })
  }

  /// Carves a circular hole of `radius` pixels centered at screen coordinate
  /// `(cx, cy)` into the overlay, revealing whatever is beneath it.
  ///
  /// A non-positive `radius` shows the full snapshot (no hole). When the hole
  /// fully covers the monitor the overlay becomes empty (fully revealed).
  pub fn set_hole(&self, cx: i32, cy: i32, radius: i32) {
    let w = self.monitor.width();
    let h = self.monitor.height();

    // SAFETY: regions are created here; `SetWindowRgn` takes ownership of the
    // final region, and the temporary ellipse is deleted explicitly.
    unsafe {
      let region: HRGN = CreateRectRgn(0, 0, w, h);
      if region.0 == 0 {
        return;
      }
      if radius > 0 {
        // Map the screen-space origin into the overlay's local space.
        let lcx = cx - self.monitor.x();
        let lcy = cy - self.monitor.y();
        let hole = CreateEllipticRgn(
          lcx - radius,
          lcy - radius,
          lcx + radius,
          lcy + radius,
        );
        if hole.0 != 0 {
          // `region` becomes `region` minus the circular `hole`.
          CombineRgn(region, region, hole, RGN_DIFF);
          let _ = DeleteObject(HGDIOBJ(hole.0));
        }
      }
      // Ownership of `region` transfers to the window; do not free it here.
      // `bRedraw = FALSE`: the snapshot is static, so only the compositor needs
      // to update the newly-exposed area, not the window's own pixels.
      SetWindowRgn(HWND(self.hwnd), region, BOOL(0));
    }
  }
}

impl Drop for NativeIrisOverlay {
  fn drop(&mut self) {
    // SAFETY: both handles were created in `create`. Destroy the window before
    // freeing the bitmap it referenced.
    unsafe {
      let _ = DestroyWindow(HWND(self.hwnd));
      if self.bitmap != 0 {
        let _ = DeleteObject(HGDIOBJ(self.bitmap));
      }
    }
  }
}
