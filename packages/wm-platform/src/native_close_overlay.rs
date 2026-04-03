use windows::{
  core::w,
  Win32::{
    Foundation::{COLORREF, HWND, RECT},
    Graphics::Dwm::{
      DwmRegisterThumbnail, DwmUnregisterThumbnail,
      DwmUpdateThumbnailProperties, DWM_THUMBNAIL_PROPERTIES,
      DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION, DWM_TNP_RECTSOURCE,
      DWM_TNP_SOURCECLIENTAREAONLY, DWM_TNP_VISIBLE,
    },
    UI::WindowsAndMessaging::{
      CreateWindowExW, DestroyWindow, IsWindow, SetLayeredWindowAttributes,
      SetWindowPos, LWA_ALPHA, SWP_NOACTIVATE, SWP_NOCOPYBITS,
      SWP_NOMOVE, SWP_NOSENDCHANGING, SWP_NOSIZE, SWP_NOZORDER,
      SWP_SHOWWINDOW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
      WS_EX_TRANSPARENT, WS_POPUP,
    },
  },
};

use crate::{native_surrogate::ensure_class_registered, Rect};

/// Lightweight overlay window used for window-close animations.
///
/// A DWM thumbnail of the closing window is rendered inside the overlay and
/// scales as the overlay contracts. `WS_EX_LAYERED` is used for per-frame
/// alpha control so the window can fade out simultaneously.
///
/// When the animation finishes the overlay is dropped, which unregisters the
/// thumbnail and destroys the window.
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct NativeCloseOverlay {
  /// Handle to the overlay window.
  hwnd: isize,
  /// DWM thumbnail handle, or `0` if registration failed or was released.
  thumbnail: isize,
  /// Width of the source window at the time `begin` was called.
  src_w: i32,
  /// Height of the source window at the time `begin` was called.
  src_h: i32,
}

impl NativeCloseOverlay {
  /// Creates a close overlay positioned at `source_rect`, displaying live
  /// content from `source_hwnd` via a DWM thumbnail.
  ///
  /// Returns an error if window creation fails or the thumbnail cannot be
  /// registered (e.g. the source window has already been destroyed).
  pub fn begin(
    source_hwnd: HWND,
    source_rect: &Rect,
  ) -> crate::Result<Self> {
    // SAFETY: `IsWindow` is safe to call with any `HWND` value.
    if !unsafe { IsWindow(source_hwnd).as_bool() } {
      return Err(crate::Error::Platform(
        "Source window no longer exists.".to_string(),
      ));
    }

    ensure_class_registered();

    let src_w = source_rect.width();
    let src_h = source_rect.height();

    // SAFETY: Class name is the static literal registered by `NativeSurrogate`.
    let hwnd = unsafe {
      CreateWindowExW(
        WS_EX_LAYERED
          | WS_EX_NOACTIVATE
          | WS_EX_TOOLWINDOW
          | WS_EX_TRANSPARENT,
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
        "Failed to create close overlay window.".to_string(),
      ));
    }

    // Start fully opaque; opacity is reduced each frame via `update`.
    // SAFETY: `hwnd` is a valid window handle created above.
    unsafe { SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA) }?;

    // Register the thumbnail; fail if the source is already destroyed.
    // SAFETY: Both handles are valid top-level windows.
    let thumbnail = unsafe { DwmRegisterThumbnail(hwnd, source_hwnd) }
      .map_err(|e| crate::Error::Platform(e.to_string()))?;

    let pinned_src =
      RECT { left: 0, top: 0, right: src_w, bottom: src_h };

    let props = DWM_THUMBNAIL_PROPERTIES {
      dwFlags: DWM_TNP_RECTDESTINATION
        | DWM_TNP_RECTSOURCE
        | DWM_TNP_OPACITY
        | DWM_TNP_VISIBLE
        | DWM_TNP_SOURCECLIENTAREAONLY,
      rcDestination: pinned_src,
      rcSource: pinned_src,
      opacity: 255,
      fVisible: true.into(),
      fSourceClientAreaOnly: true.into(),
      ..Default::default()
    };

    // SAFETY: `thumbnail` is a valid handle from `DwmRegisterThumbnail`.
    if unsafe {
      DwmUpdateThumbnailProperties(thumbnail, &raw const props)
    }
    .is_err()
    {
      // SAFETY: Same handle; unregister on failure.
      unsafe { let _ = DwmUnregisterThumbnail(thumbnail); };
      // SAFETY: `hwnd` is valid; destroy to avoid leaking the window.
      unsafe { let _ = DestroyWindow(hwnd); };
      return Err(crate::Error::Platform(
        "Failed to configure close overlay thumbnail.".to_string(),
      ));
    }

    // Place the overlay above `source_hwnd` and show it without activating.
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
      src_w,
      src_h,
    })
  }

  /// Updates the overlay position/size and opacity for the current animation
  /// frame.
  ///
  /// `rcDestination` is updated to fill the new rect so the DWM thumbnail
  /// scales with the overlay as it contracts.
  pub fn update(&mut self, rect: &Rect, alpha: u8) -> crate::Result<()> {
    // SAFETY: `HWND(self.hwnd)` is valid until `Drop`.
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

    // SAFETY: `HWND(self.hwnd)` is valid until `Drop`.
    unsafe {
      SetLayeredWindowAttributes(
        HWND(self.hwnd),
        COLORREF(0),
        alpha,
        LWA_ALPHA,
      )
    }?;

    if self.thumbnail != 0 {
      let props = DWM_THUMBNAIL_PROPERTIES {
        dwFlags: DWM_TNP_RECTDESTINATION,
        rcDestination: RECT {
          left: 0,
          top: 0,
          right: rect.width(),
          bottom: rect.height(),
        },
        ..Default::default()
      };

      // SAFETY: `self.thumbnail` is a valid handle until it is cleared.
      if let Err(err) = unsafe {
        DwmUpdateThumbnailProperties(self.thumbnail, &raw const props)
      } {
        tracing::warn!("Close overlay thumbnail update failed: {err}.");
        self.thumbnail = 0;
      }
    }

    Ok(())
  }

  /// Returns the original source width captured at `begin` time.
  pub fn src_width(&self) -> i32 {
    self.src_w
  }

  /// Returns the original source height captured at `begin` time.
  pub fn src_height(&self) -> i32 {
    self.src_h
  }
}

impl Drop for NativeCloseOverlay {
  fn drop(&mut self) {
    // SAFETY: Both handles are valid until explicitly destroyed here.
    // The thumbnail must be unregistered before the destination window is
    // destroyed.
    unsafe {
      if self.thumbnail != 0 {
        let _ = DwmUnregisterThumbnail(self.thumbnail);
      }
      let _ = DestroyWindow(HWND(self.hwnd));
    }
  }
}
