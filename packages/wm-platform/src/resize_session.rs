use windows::Win32::{
  Foundation::HWND,
  UI::WindowsAndMessaging::{
    IsWindow, SetWindowPos, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOCOPYBITS,
    SWP_NOSENDCHANGING, SWP_NOZORDER,
  },
};

use crate::{NativeSurrogate, Rect, SurrogateBackdrop};

/// Tracks a single window's resize/move animation and manages its surrogate
/// overlay.
///
/// `ResizeSession` ensures the surrogate always outlives the final
/// `SetWindowPos` sent to the real window. On `WmState` drop, [`commit`] is
/// called on all active sessions so no window is left at an intermediate
/// position after a crash or forced exit.
///
/// [`commit`]: ResizeSession::commit
///
/// # Platform-specific
///
/// Only available on Windows.
pub struct ResizeSession {
  /// Raw handle to the real app window. Stored as `isize` to avoid `Send`
  /// issues with windows-rs handle types. Set to `0` by `pre_commit` when
  /// the window has been destroyed.
  hwnd: isize,
  /// Final target rect for the real window.
  target_rect: Rect,
  /// Surrogate overlay; `None` if creation failed.
  surrogate: Option<NativeSurrogate>,
}

impl ResizeSession {
  /// Creates a resize session with a DWM surrogate overlay.
  ///
  /// The surrogate displays the real window's live content via a
  /// `DwmRegisterThumbnail` with a pinned `rcSource` equal to the original
  /// window size, so the thumbnail is never scaled regardless of how the
  /// source window is resized underneath. Acrylic fills the area around it as
  /// the surrogate grows toward `target_rect`.
  ///
  /// When the surrogate is successfully created the real window is immediately
  /// repositioned to `target_rect` while hidden beneath the overlay. Because
  /// `rcSource` is pinned, this does not affect the thumbnail content. The
  /// window renders at the correct final size for the entire animation so
  /// uncloaking at animation end requires no repaint and produces no flicker.
  ///
  /// When surrogate creation fails the session is returned without one — the
  /// animation falls back to direct window repositioning every frame.
  pub fn begin(
    hwnd: HWND,
    source_rect: &Rect,
    target_rect: &Rect,
    backdrop: &SurrogateBackdrop,
  ) -> crate::Result<Self> {
    let surrogate =
      match NativeSurrogate::create(hwnd, source_rect, backdrop) {
        Ok(s) => Some(s),
        Err(err) => {
          tracing::warn!(
            "Failed to create surrogate: {err}. Falling back to direct \
             animation."
          );
          None
        }
      };

    // Pre-position the real window at the target rect while it is covered by
    // the surrogate. The pinned `rcSource` on the thumbnail means this resize
    // does not affect the displayed content. By animation end the window will
    // have already rendered at the correct size, making uncloak flicker-free.
    if surrogate.is_some() {
      let r = target_rect;

      // SAFETY: `hwnd` is a valid top-level window handle. `SWP_NOZORDER`
      // makes `hWndInsertAfter` irrelevant. `SWP_ASYNCWINDOWPOS` posts to
      // the window's message queue without blocking our thread.
      unsafe {
        let _ = SetWindowPos(
          hwnd,
          HWND(0),
          r.x(),
          r.y(),
          r.width(),
          r.height(),
          SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOZORDER,
        );
      }
    }

    Ok(Self {
      hwnd: hwnd.0,
      target_rect: target_rect.clone(),
      surrogate,
    })
  }

  /// Whether a surrogate overlay is currently active for this session.
  pub fn has_surrogate(&self) -> bool {
    self.surrogate.is_some()
  }

  /// Updates the surrogate to the current animation frame position.
  pub fn update(&mut self, current_rect: &Rect) {
    if let Some(surrogate) = &mut self.surrogate {
      if let Err(err) = surrogate.update(current_rect) {
        tracing::warn!("Surrogate update failed: {err}.");
      }
    }
  }

  /// Snaps the surrogate to the final target rect in preparation for
  /// `platform_sync` to uncloak the real window.
  ///
  /// Checks `IsWindow` and nullifies the stored handle if the window has been
  /// destroyed mid-animation, so that [`commit`] skips the `SetWindowPos`
  /// call.
  ///
  /// [`commit`]: ResizeSession::commit
  pub fn pre_commit(&mut self) {
    // SAFETY: `IsWindow` is safe to call with any `HWND` value.
    if !unsafe { IsWindow(HWND(self.hwnd)).as_bool() } {
      self.hwnd = 0;
      return;
    }

    if let Some(surrogate) = &mut self.surrogate {
      if let Err(err) = surrogate.update(&self.target_rect.clone()) {
        tracing::warn!("Surrogate pre-commit update failed: {err}.");
      }
    }
  }

  /// Moves the real window to its final target rect and destroys the
  /// surrogate.
  ///
  /// Intended as a cleanup path (e.g. on `WmState::Drop`) to prevent windows
  /// from being left at intermediate animation positions after a crash or
  /// forced exit. Checks `IsWindow` before calling `SetWindowPos` to handle
  /// windows destroyed mid-animation.
  ///
  /// For normal animation completion, `platform_sync` calls
  /// `reposition_window` which handles the full `SetWindowPos` path
  /// including maximize/restore handling; this method is a best-effort
  /// fallback only.
  pub fn commit(mut self) -> crate::Result<()> {
    // Destroy the surrogate before moving the real window so the overlay
    // never outlives the final position update.
    drop(self.surrogate.take());

    if self.hwnd == 0 {
      return Ok(());
    }

    // SAFETY: `IsWindow` is safe to call with any `HWND` value.
    if !unsafe { IsWindow(HWND(self.hwnd)).as_bool() } {
      return Ok(());
    }

    let r = &self.target_rect;

    // SAFETY: `HWND(self.hwnd)` is valid (verified above). With
    // `SWP_NOZORDER` set, `hWndInsertAfter` (`HWND(0)`) is ignored per
    // the Win32 documentation.
    unsafe {
      SetWindowPos(
        HWND(self.hwnd),
        HWND(0),
        r.x(),
        r.y(),
        r.width(),
        r.height(),
        SWP_NOACTIVATE | SWP_NOCOPYBITS | SWP_NOSENDCHANGING | SWP_NOZORDER,
      )
    }?;

    Ok(())
  }
}
