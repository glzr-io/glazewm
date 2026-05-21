use windows::Win32::{
  Foundation::{HWND, RECT},
  Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
  UI::WindowsAndMessaging::{
    GetWindowRect, IsWindow, SetWindowPos, SWP_NOACTIVATE, SWP_NOSENDCHANGING,
    SWP_NOZORDER,
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
  /// `true` when no dimension shrinks (target >= source in both width and
  /// height). Curtain-reveal mode.
  ///
  /// Growing sessions use a curtain-reveal: thumbnail registered at target
  /// dimensions; cloaked window pre-positioned asynchronously so DWM captures
  /// correctly-sized content. Mixed/shrinking sessions use clip/wipe: thumbnail
  /// at source dimensions, real window stays at source until `pre_commit`.
  is_growing: bool,
  /// When `true`, each frame animates the DWM thumbnail `rcDestination`
  /// toward/away from the surrogate center instead of repositioning the
  /// surrogate window. Used for zoom-in (open) and zoom-out (close) effects.
  pub zoom: bool,
}

impl ResizeSession {
  /// Creates a resize session with a DWM surrogate overlay.
  ///
  /// The surrogate is positioned at the **logical** rect (physical minus
  /// invisible border) so it does not overlap the configured window gap.
  ///
  /// For **shrinking** animations (`target` narrower/shorter than `source`):
  /// the thumbnail is registered at `source_rect` dimensions, filling the
  /// whole surrogate initially. As the surrogate shrinks the thumbnail edge is
  /// clipped — a wipe effect with no content distortion and no timing
  /// dependency on the real window re-rendering. The real window stays at
  /// `source_rect` until [`pre_commit`] moves it synchronously at animation
  /// end.
  ///
  /// For **growing** animations (no dimension shrinks — target >= source in
  /// both axes): the thumbnail is registered at `target_rect` dimensions. The
  /// caller (platform_sync's Frozen branch) asynchronously pre-positions the
  /// cloaked real window at the target rect immediately after cloaking so DWM
  /// can capture the correctly-sized content during the curtain-reveal.
  ///
  /// For **mixed** animations (one axis grows, one shrinks — e.g. side-by-side
  /// to top-bottom layout): the thumbnail is registered at `source_rect`
  /// dimensions (clip/wipe). `surrogate_color` is applied as a backdrop so the
  /// area uncovered by the thumbnail shows the configured color rather than
  /// exposing the desktop.
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
    initially_visible: bool,
  ) -> crate::Result<Self> {
    let border_inset = compute_border_inset(hwnd);

    // Curtain-reveal when no dimension shrinks (target >= source in both axes).
    let is_growing = target_rect.width() >= source_rect.width()
      && target_rect.height() >= source_rect.height();

    // Growing: thumbnail at target dimensions for curtain-reveal.
    // Shrinking/mixed: thumbnail at source dimensions for clip/wipe.
    let thumbnail_rect = if is_growing { target_rect } else { source_rect };

    // Mixed: one axis grows, one shrinks. Apply the backdrop color so the area
    // the thumbnail cannot cover shows the configured color instead of the desktop.
    let is_mixed = !is_growing
      && (target_rect.width() > source_rect.width()
        || target_rect.height() > source_rect.height());
    let backdrop_color = if is_mixed { surrogate_color } else { None };

    let surrogate = match NativeSurrogate::create(
      hwnd,
      source_rect,
      thumbnail_rect,
      backdrop_color,
      effect_opacity,
      initially_visible,
      border_inset,
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

    Ok(Self {
      hwnd: hwnd.0,
      target_rect: target_rect.clone(),
      surrogate,
      border_inset,
      effect_opacity,
      is_growing,
      zoom: false,
    })
  }

  /// Returns `true` when no dimension shrinks (curtain-reveal mode).
  ///
  /// Used by `platform_sync` to decide whether to asynchronously pre-position
  /// the cloaked real window at the target rect immediately after cloaking.
  pub fn is_growing(&self) -> bool {
    self.is_growing
  }

  /// Whether a surrogate overlay with a valid DWM thumbnail is active.
  ///
  /// Returns `false` when surrogate creation failed, or when the surrogate
  /// window exists but thumbnail registration failed (e.g. elevated/UWP
  /// source windows). Callers use this to decide whether to freeze the real
  /// window behind the surrogate or fall back to direct repositioning.
  pub fn has_surrogate(&self) -> bool {
    self.surrogate.as_ref().map_or(false, |s| s.has_thumbnail())
  }

  /// Makes the surrogate visible.
  ///
  /// Used after creating the surrogate with `initially_visible = false` to
  /// reveal it once the real window has been cloaked.
  pub fn show(&mut self) {
    if let Some(ref mut surrogate) = self.surrogate {
      surrogate.set_visible(true);
    }
  }

  /// Animates the DWM thumbnail `rcDestination` toward/away from center.
  ///
  /// `progress` is the eased animation progress (0.0 = zero-size, 1.0 = full
  /// surrogate). Used for zoom-in (open) and zoom-out (close) effects. The
  /// surrogate window itself stays fixed; only the thumbnail rect animates.
  pub fn update_zoom_fade(&mut self, progress: f32, opacity: u8) {
    let Some(ref mut surrogate) = self.surrogate else {
      return;
    };
    let logical = to_logical(&self.target_rect, &self.border_inset);
    let w = logical.width();
    let h = logical.height();
    let half_w = (w as f32 / 2.0 * progress).round() as i32;
    let half_h = (h as f32 / 2.0 * progress).round() as i32;
    if half_w <= 0 || half_h <= 0 {
      surrogate.set_visible(false);
    } else {
      let cx = w / 2;
      let cy = h / 2;
      surrogate.set_thumbnail_rects(
        RECT { left: 0, top: 0, right: w, bottom: h },
        RECT {
          left: cx - half_w,
          top: cy - half_h,
          right: cx + half_w,
          bottom: cy + half_h,
        },
      );
      surrogate.set_visible(true);
    }
    surrogate.set_window_opacity(opacity);
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

  /// Updates the surrogate, clamping its visible area to `monitor_rect`.
  ///
  /// When `current_rect` extends outside `monitor_rect`, the surrogate is
  /// constrained to the intersection and the DWM thumbnail `rcSource` and
  /// `rcDestination` are adjusted to show only the visible slice — matching
  /// the approach used by `WorkspaceSurrogate`. Hides the surrogate when
  /// the rect is fully off-screen.
  pub fn update_clipped(
    &mut self,
    current_rect: &Rect,
    monitor_rect: &Rect,
    opacity: u8,
  ) {
    let Some(surrogate) = &mut self.surrogate else {
      return;
    };

    let logical = to_logical(current_rect, &self.border_inset);

    let vis_left = logical.x().max(monitor_rect.x());
    let vis_top = logical.y().max(monitor_rect.y());
    let vis_right = (logical.x() + logical.width())
      .min(monitor_rect.x() + monitor_rect.width());
    let vis_bottom = (logical.y() + logical.height())
      .min(monitor_rect.y() + monitor_rect.height());

    if vis_left >= vis_right || vis_top >= vis_bottom {
      surrogate.set_visible(false);
      return;
    }

    let src_left = vis_left - logical.x();
    let src_top = vis_top - logical.y();
    let constrained_w = vis_right - vis_left;
    let constrained_h = vis_bottom - vis_top;

    surrogate.set_thumbnail_rects(
      RECT { left: src_left, top: src_top, right: src_left + constrained_w, bottom: src_top + constrained_h },
      RECT { left: 0, top: 0, right: constrained_w, bottom: constrained_h },
    );

    let constrained = Rect::from_xy(vis_left, vis_top, constrained_w, constrained_h);
    if let Err(err) = surrogate.reposition(&constrained) {
      tracing::warn!("Surrogate clipped update failed: {err}.");
    }
    surrogate.set_window_opacity(opacity);
    surrogate.set_visible(true);
  }

  /// Redirects the session to a new target rect while the surrogate is still
  /// active.
  ///
  /// For **shrinking** sessions: only updates the stored `target_rect`. The
  /// single synchronous move happens in [`pre_commit`] once the animation
  /// reaches its final destination.
  ///
  /// For **growing** sessions: additionally sends a synchronous `SetWindowPos`
  /// to pre-position the cloaked real window at `new_target` and re-registers
  /// the DWM thumbnail at the new target dimensions. This ensures the
  /// curtain-reveal correctly covers the newly expanded area when a second
  /// window close redirects an in-flight animation to a larger target.
  ///
  /// [`pre_commit`]: ResizeSession::pre_commit
  pub fn update_target(&mut self, new_target: &Rect) {
    self.target_rect = new_target.clone();

    if !self.is_growing || self.hwnd == 0 {
      return;
    }

    // SAFETY: Window is cloaked when `update_target` is called (invoked from
    // `start_animation_if_needed` during an active Frozen animation), so this
    // `SetWindowPos` is invisible to the user.
    unsafe {
      let _ = SetWindowPos(
        HWND(self.hwnd),
        HWND(0),
        new_target.x(),
        new_target.y(),
        new_target.width(),
        new_target.height(),
        SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_NOZORDER,
      );
    }

    if let Some(surrogate) = &mut self.surrogate {
      let logical = to_logical(new_target, &self.border_inset);
      surrogate.reregister_thumbnail(
        HWND(self.hwnd),
        logical.width(),
        logical.height(),
        self.border_inset,
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
