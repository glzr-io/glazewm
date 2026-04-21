use windows::Win32::{
  Foundation::{BOOL, HWND},
  Graphics::Gdi::{CreateRectRgn, SetWindowRgn},
  UI::WindowsAndMessaging::{
    SetWindowPos, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOZORDER,
  },
};

use crate::{Color, NativeSurrogate, Rect};

/// Surrogate overlay for a single window participating in a workspace-switch
/// slide animation.
///
/// Unlike [`ResizeSession`], the surrogate translates the captured window
/// content across the monitor each frame rather than revealing it via a
/// resize. Both outgoing and incoming windows slide together so the whole
/// workspace moves as a single panel, similar to Hyprland.
///
/// [`ResizeSession`]: crate::ResizeSession
pub struct WorkspaceSurrogate {
  inner: Option<NativeSurrogate>,
  /// Final screen rect of the window (target position for incoming, current
  /// position for outgoing).
  pub rect: Rect,
}

impl WorkspaceSurrogate {
  /// Creates a surrogate for an outgoing workspace window at its current
  /// position.
  ///
  /// Call [`show_initial`] immediately after creation to make it visible
  /// before the real window is cloaked, avoiding a blank frame.
  ///
  /// [`show_initial`]: WorkspaceSurrogate::show_initial
  pub fn new_outgoing(
    hwnd: HWND,
    rect: &Rect,
    color: Option<&Color>,
  ) -> crate::Result<Self> {
    let inner =
      NativeSurrogate::create(hwnd, rect, rect, color, false)?;
    Ok(Self {
      inner: Some(inner),
      rect: rect.clone(),
    })
  }

  /// Creates a surrogate for an incoming workspace window.
  ///
  /// The surrogate is positioned off-screen in the direction of the switch
  /// so that it slides in smoothly from the monitor edge. The real window is
  /// pre-positioned at `rect` asynchronously so the DWM thumbnail shows the
  /// correct final-size content for the entire animation.
  pub fn new_incoming(
    hwnd: HWND,
    rect: &Rect,
    direction: i32,
    monitor_width: i32,
    color: Option<&Color>,
  ) -> crate::Result<Self> {
    // Pre-position the real window at its final rect so the thumbnail shows
    // the correct size. Async so we don't block on the target's message queue.
    //
    // SAFETY: `hwnd` is a valid top-level window. `SWP_NOZORDER` makes the
    // insert-after parameter irrelevant. `SWP_ASYNCWINDOWPOS` posts the move
    // to the window's message queue without blocking the caller.
    unsafe {
      let _ = SetWindowPos(
        hwnd,
        HWND(0),
        rect.x(),
        rect.y(),
        rect.width(),
        rect.height(),
        SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOZORDER,
      );
    }

    let start_x = rect.x() + direction * monitor_width;
    let start_rect =
      Rect::from_xy(start_x, rect.y(), rect.width(), rect.height());
    let inner =
      NativeSurrogate::create(hwnd, &start_rect, rect, color, false)?;

    Ok(Self {
      inner: Some(inner),
      rect: rect.clone(),
    })
  }

  /// Shows the surrogate at full opacity at its initial (current) position.
  ///
  /// Must be called on outgoing surrogates before the real window is cloaked
  /// so there is no blank frame between cloaking and the first slide tick.
  pub fn show_initial(&mut self) {
    if let Some(ref mut inner) = self.inner {
      let _ = inner.update(&self.rect, 255);
    }
  }

  /// Advances the surrogate to the given `eased_progress` (0.0 → 1.0).
  ///
  /// Outgoing windows translate off-screen opposite to `direction`; incoming
  /// windows translate from off-screen to their final position. Both are
  /// clipped to the monitor bounds (`monitor_x .. monitor_x + monitor_width`)
  /// so surrogates never spill onto adjacent monitors.
  pub fn update_slide(
    &mut self,
    eased_progress: f32,
    is_incoming: bool,
    direction: i32,
    monitor_x: i32,
    monitor_width: i32,
  ) {
    let Some(ref mut inner) = self.inner else {
      return;
    };

    // Compute the per-frame x offset.
    // Incoming: start at +direction*monitor_width, end at 0.
    // Outgoing: start at 0, end at -direction*monitor_width.
    let offset = if is_incoming {
      (direction as f32 * monitor_width as f32 * (1.0 - eased_progress))
        as i32
    } else {
      (-direction as f32 * monitor_width as f32 * eased_progress) as i32
    };

    let current_x = self.rect.x() + offset;
    let current_rect = Rect::from_xy(
      current_x,
      self.rect.y(),
      self.rect.width(),
      self.rect.height(),
    );

    // Compute the visible strip within the monitor bounds.
    let monitor_right = monitor_x + monitor_width;
    let vis_left = current_x.max(monitor_x);
    let vis_right = (current_x + self.rect.width()).min(monitor_right);

    if vis_left >= vis_right {
      // Completely off-screen: shrink to a 1×1 pixel outside the monitor so
      // the surrogate is invisible without destroying it.
      let _ = inner.update(
        &Rect::from_xy(monitor_x - 2, self.rect.y(), 1, 1),
        0,
      );
      return;
    }

    let _ = inner.update(&current_rect, 255);

    // Clip the surrogate to the monitor bounds (window-local coordinates) so
    // it does not render on adjacent monitors during the slide.
    let clip_x = vis_left - current_x;
    let clip_right = clip_x + (vis_right - vis_left);

    // SAFETY: Coordinates are valid integers. After a successful
    // `SetWindowRgn` call the system owns the HRGN; `DeleteObject` must not
    // be called on it. If `SetWindowRgn` fails we leak a small HRGN, which is
    // acceptable (creation failure is itself rare and non-fatal).
    unsafe {
      let hrgn = CreateRectRgn(clip_x, 0, clip_right, self.rect.height());
      if !hrgn.is_invalid() {
        let _ = SetWindowRgn(inner.hwnd(), hrgn, BOOL(1));
      }
    }
  }
}
