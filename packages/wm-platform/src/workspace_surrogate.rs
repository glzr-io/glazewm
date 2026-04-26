use windows::Win32::{
  Foundation::{HWND, RECT},
  Graphics::Dwm::{
    DwmUpdateThumbnailProperties, DWM_THUMBNAIL_PROPERTIES,
    DWM_TNP_OPACITY, DWM_TNP_RECTDESTINATION, DWM_TNP_RECTSOURCE,
    DWM_TNP_SOURCECLIENTAREAONLY, DWM_TNP_VISIBLE,
  },
};

use crate::{Color, NativeSurrogate, Rect};

/// Surrogate overlay for a single window participating in a workspace-switch
/// slide animation.
///
/// Both outgoing and incoming windows slide together so the whole workspace
/// moves as a single panel, similar to Hyprland. The surrogate window is
/// always constrained to the monitor bounds — it is hidden (via
/// `SW_HIDE`) when fully off-screen and shown only when it has a visible
/// area. The DWM thumbnail's `rcSource` is updated each frame to display
/// the correct content slice, so the surrogate window never extends onto
/// an adjacent monitor.
pub struct WorkspaceSurrogate {
  inner: Option<NativeSurrogate>,
  /// Final screen rect of the window (target position for incoming, current
  /// screen rect for outgoing).
  pub rect: Rect,
  /// Whether the surrogate is currently shown.
  is_visible: bool,
}

impl WorkspaceSurrogate {
  /// Creates a surrogate for an outgoing workspace window at its current
  /// position.
  ///
  /// The surrogate is created hidden. Call [`show_initial`] before cloaking
  /// the real window to make the surrogate visible without a blank frame.
  ///
  /// [`show_initial`]: WorkspaceSurrogate::show_initial
  pub fn new_outgoing(
    hwnd: HWND,
    rect: &Rect,
    color: Option<&Color>,
  ) -> crate::Result<Self> {
    let inner =
      NativeSurrogate::create(hwnd, rect, rect, color, false, false, false)?;
    Ok(Self {
      inner: Some(inner),
      rect: rect.clone(),
      is_visible: false,
    })
  }

  /// Creates a surrogate for an incoming workspace window.
  ///
  /// The surrogate is created hidden — it will be shown by [`update_slide`]
  /// as soon as it enters the monitor's visible area. The real window is
  /// left at its current position and cloaked by the caller; the DWM
  /// thumbnail captures its compositor content regardless of position.
  /// Passing `initially_visible: false` to [`NativeSurrogate::create`]
  /// prevents a one-frame flash of the surrogate at the target rect before
  /// the animation has started.
  ///
  /// [`update_slide`]: WorkspaceSurrogate::update_slide
  /// [`NativeSurrogate::create`]: crate::NativeSurrogate::create
  pub fn new_incoming(
    hwnd: HWND,
    rect: &Rect,
    color: Option<&Color>,
  ) -> crate::Result<Self> {
    // Create the surrogate at the final rect but keep it hidden. The first
    // `update_slide` call will position and reveal it when it enters the
    // current monitor's bounds.
    let inner =
      NativeSurrogate::create(hwnd, rect, rect, color, false, false, false)?;

    Ok(Self {
      inner: Some(inner),
      rect: rect.clone(),
      is_visible: false,
    })
  }

  /// Shows the surrogate at full opacity covering its full rect.
  ///
  /// Must be called on outgoing surrogates before the real window is cloaked
  /// so there is no blank frame between cloaking and the first slide tick.
  pub fn show_initial(&mut self) {
    let Some(ref mut inner) = self.inner else {
      return;
    };
    let _ = inner.update(&self.rect, 255);
    inner.set_visible(true);
    self.is_visible = true;
  }

  /// Advances the surrogate to `eased_progress` (0.0 → 1.0).
  ///
  /// The surrogate window is constrained to `[monitor_x, monitor_x +
  /// monitor_width]` at all times — it is hidden when fully off-screen and
  /// shown only for the visible strip. The DWM thumbnail's `rcSource` is
  /// updated to show the correct content slice so no stretching occurs and
  /// no pixels appear outside the current monitor.
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
    // Incoming: start at +direction*monitor_width offset, end at 0.
    // Outgoing: start at 0, end at -direction*monitor_width offset.
    let offset = if is_incoming {
      (direction as f32 * monitor_width as f32 * (1.0 - eased_progress))
        as i32
    } else {
      (-direction as f32 * monitor_width as f32 * eased_progress) as i32
    };

    let current_x = self.rect.x() + offset;
    let monitor_right = monitor_x + monitor_width;

    // Visible strip of this window on the current monitor.
    let vis_left = current_x.max(monitor_x);
    let vis_right = (current_x + self.rect.width()).min(monitor_right);

    if vis_left >= vis_right {
      // Completely off-screen: hide to prevent rendering on adjacent monitors.
      if self.is_visible {
        inner.set_visible(false);
        self.is_visible = false;
      }
      return;
    }

    let constrained_w = vis_right - vis_left;

    // The portion of the source window (in window-local pixels) that maps
    // to the visible screen strip.
    let src_left = vis_left - current_x;
    let src_right = src_left + constrained_w;

    // Update the DWM thumbnail: show only the visible slice of source
    // content, mapped 1:1 onto the constrained surrogate rect.
    let thumbnail = inner.thumbnail();
    if thumbnail != 0 {
      let props = DWM_THUMBNAIL_PROPERTIES {
        dwFlags: DWM_TNP_RECTSOURCE
          | DWM_TNP_RECTDESTINATION
          | DWM_TNP_OPACITY
          | DWM_TNP_VISIBLE
          | DWM_TNP_SOURCECLIENTAREAONLY,
        rcSource: RECT {
          left: src_left,
          top: 0,
          right: src_right,
          bottom: self.rect.height(),
        },
        rcDestination: RECT {
          left: 0,
          top: 0,
          right: constrained_w,
          bottom: self.rect.height(),
        },
        opacity: 255,
        fVisible: true.into(),
        fSourceClientAreaOnly: false.into(),
        ..Default::default()
      };
      // SAFETY: `thumbnail` is a valid handle. `props` is stack-allocated
      // and remains live for the duration of this call.
      unsafe {
        let _ =
          DwmUpdateThumbnailProperties(thumbnail, &raw const props);
      }
    }

    // Position the surrogate window at the constrained (monitor-clamped) rect.
    let constrained_rect = Rect::from_xy(
      vis_left,
      self.rect.y(),
      constrained_w,
      self.rect.height(),
    );
    let _ = inner.update(&constrained_rect, 255);

    if !self.is_visible {
      inner.set_visible(true);
      self.is_visible = true;
    }
  }
}
