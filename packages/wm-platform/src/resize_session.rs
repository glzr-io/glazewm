use windows::Win32::{
  Foundation::{HWND, RECT},
  Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
  UI::WindowsAndMessaging::{
    GetWindowRect, IsWindow, SetWindowPos, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE,
    SWP_NOSENDCHANGING, SWP_NOZORDER,
  },
};

use crate::{native_surrogate::to_logical, Color, NativeSurrogate, Rect};

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
  /// Final target rect for the real window (physical, including invisible
  /// border).
  target_rect: Rect,
  /// Surrogate overlay; `None` if creation failed.
  surrogate: Option<NativeSurrogate>,
  /// Invisible border insets (left, top, right, bottom) of the source window
  /// in physical pixels. Applied when converting physical rects to the logical
  /// (visible-content) rects that the surrogate is sized to.
  border_inset: RECT,
  /// DWM thumbnail opacity (0–255) from the window-effects config.
  ///
  /// Used as the surrogate opacity when the animation has no per-frame fade
  /// component, so the thumbnail matches the real window's `SetLayeredWindowAttributes`
  /// opacity throughout the move/resize.
  pub effect_opacity: u8,
  /// Whether the target is smaller than the source in at least one dimension.
  ///
  /// When `true`, the real window is kept at its source position throughout the
  /// animation and only moved synchronously in [`pre_commit`]. This prevents a
  /// visible jump (e.g. when a new window spawns and adjacent windows shrink to
  /// make room). Growing windows still pre-position via `SWP_ASYNCWINDOWPOS` so
  /// the window can render at its final size before the surrogate drops.
  ///
  /// [`pre_commit`]: ResizeSession::pre_commit
  is_shrinking: bool,
}

impl ResizeSession {
  /// Creates a resize session with a DWM surrogate overlay.
  ///
  /// The surrogate is positioned at the **logical** rect (physical minus
  /// invisible border) so it does not overlap the configured window gap.
  /// The DWM thumbnail is pinned to the **target** logical dimensions so it
  /// always shows the final rendered content.
  ///
  /// For growing windows the real window is immediately pre-positioned to
  /// `target_rect` via `SWP_ASYNCWINDOWPOS` so it renders at its final size
  /// before the surrogate drops. For shrinking windows the real window stays
  /// at `source_rect` and is only moved synchronously in [`pre_commit`], so
  /// it never jumps to a smaller position while another window is on top.
  ///
  /// When surrogate creation fails the session is returned without one — the
  /// animation falls back to direct window repositioning every frame.
  ///
  /// [`pre_commit`]: ResizeSession::pre_commit
  pub fn begin(
    hwnd: HWND,
    source_rect: &Rect,
    target_rect: &Rect,
    surrogate_color: Option<&Color>,
    effect_opacity: u8,
  ) -> crate::Result<Self> {
    let border_inset = compute_border_inset(hwnd);

    // Shrinking: target is smaller than source in at least one dimension.
    // Growing windows are pre-positioned to target early so the window can
    // render at its final size before the surrogate drops (curtain-reveal).
    // Shrinking windows stay at source to avoid a visible jump (the surrogate
    // collapses on top of the real window; the final SetWindowPos fires
    // synchronously in `pre_commit`).
    let is_shrinking = target_rect.width() < source_rect.width()
      || target_rect.height() < source_rect.height();

    let surrogate = match NativeSurrogate::create(
      hwnd,
      source_rect,
      target_rect,
      surrogate_color,
      effect_opacity,
      true,
      border_inset,
      is_shrinking,
    ) {
      Ok(s) => Some(s),
      Err(err) => {
        tracing::warn!(
          "Failed to create surrogate: {err}. Falling back to direct \
           animation."
        );
        None
      }
    };

    // Pre-position the real window at the physical target rect while it is
    // covered by the surrogate. The pinned `rcSource` on the thumbnail means
    // this resize does not affect the displayed content. By animation end the
    // window will have already rendered at the correct size, making uncloak
    // flicker-free.
    //
    // Skipped for shrinking sessions: the real window stays at source and is
    // only moved synchronously at `pre_commit`, so it never jumps to a smaller
    // position while covered by another window.
    if surrogate.is_some() && !is_shrinking {
      // SAFETY: `hwnd` is a valid top-level window handle. `SWP_NOZORDER`
      // makes `hWndInsertAfter` irrelevant. `SWP_ASYNCWINDOWPOS` posts to
      // the window's message queue without blocking our thread.
      unsafe {
        let _ = SetWindowPos(
          hwnd,
          HWND(0),
          target_rect.x(),
          target_rect.y(),
          target_rect.width(),
          target_rect.height(),
          SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOZORDER,
        );
      }
    }

    Ok(Self {
      hwnd: hwnd.0,
      target_rect: target_rect.clone(),
      surrogate,
      border_inset,
      effect_opacity,
      is_shrinking,
    })
  }

  /// Whether a surrogate overlay is currently active for this session.
  pub fn has_surrogate(&self) -> bool {
    self.surrogate.is_some()
  }

  /// Updates the surrogate to the current animation frame position and opacity.
  ///
  /// `current_rect` is the physical animated rect; it is converted to the
  /// logical rect before being applied to the surrogate window.
  ///
  /// `opacity` maps to the DWM thumbnail opacity (0 = transparent, 255 =
  /// opaque). Pass `255` for resize animations where no fade is needed.
  pub fn update(&mut self, current_rect: &Rect, opacity: u8) {
    if let Some(surrogate) = &mut self.surrogate {
      let logical = to_logical(current_rect, &self.border_inset);
      if let Err(err) = surrogate.update(&logical, opacity) {
        tracing::warn!("Surrogate update failed: {err}.");
      }
    }
  }

  /// Redirects the session to a new target rect while the surrogate is still
  /// active.
  ///
  /// Updates the stored `target_rect` and posts `SWP_ASYNCWINDOWPOS` to
  /// pre-position the real window at `new_target` so it is ready when the
  /// surrogate is eventually dropped. The surrogate thumbnail remains pinned
  /// to the original source size; only the animation destination changes.
  pub fn update_target(&mut self, new_target: &Rect) {
    self.target_rect = new_target.clone();

    // Shrinking sessions never pre-position mid-animation; the final
    // synchronous SetWindowPos in `pre_commit` is sufficient.
    if self.hwnd == 0 || self.is_shrinking {
      return;
    }

    // SAFETY: `HWND(self.hwnd)` is valid. `SWP_ASYNCWINDOWPOS` posts to the
    // window's message queue without blocking. With `SWP_NOZORDER` set,
    // `hWndInsertAfter` is ignored per the Win32 documentation.
    unsafe {
      let _ = SetWindowPos(
        HWND(self.hwnd),
        HWND(0),
        new_target.x(),
        new_target.y(),
        new_target.width(),
        new_target.height(),
        SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOZORDER,
      );
    }
  }

  /// Snaps the surrogate to the final target rect and synchronously
  /// pre-positions the real window, in preparation for `platform_sync` to
  /// uncloak it.
  ///
  /// Checks `IsWindow` and nullifies the stored handle if the window has been
  /// destroyed mid-animation, so that [`commit`] skips the `SetWindowPos`
  /// call.
  ///
  /// The synchronous `SetWindowPos` here ensures the real window is at
  /// `target_rect` before `set_cloaked(false)` fires, even when the
  /// `SWP_ASYNCWINDOWPOS` call from [`begin`] or [`update_target`] has not
  /// yet been processed by the target window's message queue.
  ///
  /// [`commit`]: ResizeSession::commit
  /// [`begin`]: ResizeSession::begin
  /// [`update_target`]: ResizeSession::update_target
  pub fn pre_commit(&mut self) {
    // SAFETY: `IsWindow` is safe to call with any `HWND` value.
    if !unsafe { IsWindow(HWND(self.hwnd)).as_bool() } {
      self.hwnd = 0;
      return;
    }

    // SAFETY: `HWND(self.hwnd)` is valid (verified above). `SWP_NOZORDER`
    // makes `hWndInsertAfter` irrelevant.
    unsafe {
      let _ = SetWindowPos(
        HWND(self.hwnd),
        HWND(0),
        self.target_rect.x(),
        self.target_rect.y(),
        self.target_rect.width(),
        self.target_rect.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOZORDER,
      );
    }

    if let Some(surrogate) = &mut self.surrogate {
      let logical = to_logical(&self.target_rect, &self.border_inset);
      if let Err(err) = surrogate.update(&logical, self.effect_opacity) {
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

    // SAFETY: `HWND(self.hwnd)` is valid (verified above). With
    // `SWP_NOZORDER` set, `hWndInsertAfter` (`HWND(0)`) is ignored per
    // the Win32 documentation.
    unsafe {
      SetWindowPos(
        HWND(self.hwnd),
        HWND(0),
        self.target_rect.x(),
        self.target_rect.y(),
        self.target_rect.width(),
        self.target_rect.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOZORDER,
      )
    }?;

    Ok(())
  }
}

/// Computes the invisible border insets of `hwnd` in physical pixels.
///
/// Windows adds a transparent resize border (~7 px on left, right, bottom;
/// none on top) outside the visible window frame. Compares `GetWindowRect`
/// with `DWMWA_EXTENDED_FRAME_BOUNDS` to obtain per-side inset values.
///
/// Returns a zeroed `RECT` if either API call fails.
fn compute_border_inset(hwnd: HWND) -> RECT {
  let mut window = RECT::default();
  let mut frame = RECT::default();

  // SAFETY: `hwnd` is a valid window handle. Both output pointers are valid
  // stack-allocated `RECT`s live for the duration of the call.
  let ok = unsafe {
    GetWindowRect(hwnd, std::ptr::from_mut(&mut window).cast()).is_ok()
      && DwmGetWindowAttribute(
        hwnd,
        DWMWA_EXTENDED_FRAME_BOUNDS,
        std::ptr::addr_of_mut!(frame).cast(),
        std::mem::size_of::<RECT>() as u32,
      )
      .is_ok()
  };

  if ok {
    RECT {
      left: (frame.left - window.left).max(0),
      top: (frame.top - window.top).max(0),
      right: (window.right - frame.right).max(0),
      bottom: (window.bottom - frame.bottom).max(0),
    }
  } else {
    RECT::default()
  }
}
