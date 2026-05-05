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
    let inner = NativeSurrogate::create(
      hwnd,
      rect,
      rect,
      color,
      false,
      false,
      RECT::default(),
    )?;
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
    let inner = NativeSurrogate::create(
      hwnd,
      rect,
      rect,
      color,
      false,
      false,
      RECT::default(),
    )?;

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

  /// Advances the surrogate along the horizontal axis to `eased_progress`
  /// (0.0 → 1.0).
  ///
  /// The surrogate window is constrained to `[monitor_x, monitor_x +
  /// monitor_width]` — hidden when fully off-screen, shown only for the
  /// visible strip. The DWM thumbnail's `rcSource` is updated to display
  /// the correct content slice, preventing stretching and spill onto
  /// adjacent monitors.
  pub fn update_slide_horizontal(
    &mut self,
    eased_progress: f32,
    is_incoming: bool,
    direction: i32,
    monitor_x: i32,
    monitor_width: i32,
  ) {
    self.slide_axis(
      eased_progress,
      is_incoming,
      direction,
      monitor_x,
      monitor_width,
      false,
    );
  }

  /// Advances the surrogate along the vertical axis to `eased_progress`
  /// (0.0 → 1.0).
  ///
  /// Behaviour mirrors [`update_slide_horizontal`] but on the y-axis:
  /// `direction = +1` means the incoming workspace slides up from below.
  ///
  /// [`update_slide_horizontal`]: WorkspaceSurrogate::update_slide_horizontal
  pub fn update_slide_vertical(
    &mut self,
    eased_progress: f32,
    is_incoming: bool,
    direction: i32,
    monitor_y: i32,
    monitor_height: i32,
  ) {
    self.slide_axis(
      eased_progress,
      is_incoming,
      direction,
      monitor_y,
      monitor_height,
      true,
    );
  }

  /// Advances the surrogate along either axis. `is_vertical = false` slides
  /// on the x-axis; `true` slides on the y-axis. `monitor_origin` and
  /// `monitor_size` are the relevant monitor bounds along the chosen axis.
  fn slide_axis(
    &mut self,
    eased_progress: f32,
    is_incoming: bool,
    direction: i32,
    monitor_origin: i32,
    monitor_size: i32,
    is_vertical: bool,
  ) {
    let Some(ref mut inner) = self.inner else {
      return;
    };

    // Incoming: start at +direction*size offset, end at 0.
    // Outgoing: start at 0, end at -direction*size offset.
    let offset = if is_incoming {
      (direction as f32 * monitor_size as f32 * (1.0 - eased_progress)) as i32
    } else {
      (-direction as f32 * monitor_size as f32 * eased_progress) as i32
    };

    // Axis-dependent dimensions.
    let (axis_pos, perp_pos, axis_size, perp_size) = if is_vertical {
      (self.rect.y(), self.rect.x(), self.rect.height(), self.rect.width())
    } else {
      (self.rect.x(), self.rect.y(), self.rect.width(), self.rect.height())
    };

    let current = axis_pos + offset;
    let monitor_end = monitor_origin + monitor_size;

    // Visible strip of this window along the sliding axis.
    let vis_start = current.max(monitor_origin);
    let vis_end = (current + axis_size).min(monitor_end);

    if vis_start >= vis_end {
      // Completely off-screen: hide to prevent rendering on adjacent monitors.
      if self.is_visible {
        inner.set_visible(false);
        self.is_visible = false;
      }
      return;
    }

    let constrained = vis_end - vis_start;
    // Source-window-local start of the visible strip.
    let src_start = vis_start - current;

    // Update the DWM thumbnail: show only the visible slice of source
    // content, mapped 1:1 onto the constrained surrogate rect.
    let thumbnail = inner.thumbnail();
    if thumbnail != 0 {
      let (rc_src, rc_dst) = if is_vertical {
        (
          RECT {
            left: 0,
            top: src_start,
            right: perp_size,
            bottom: src_start + constrained,
          },
          RECT { left: 0, top: 0, right: perp_size, bottom: constrained },
        )
      } else {
        (
          RECT {
            left: src_start,
            top: 0,
            right: src_start + constrained,
            bottom: perp_size,
          },
          RECT { left: 0, top: 0, right: constrained, bottom: perp_size },
        )
      };

      let props = DWM_THUMBNAIL_PROPERTIES {
        dwFlags: DWM_TNP_RECTSOURCE
          | DWM_TNP_RECTDESTINATION
          | DWM_TNP_OPACITY
          | DWM_TNP_VISIBLE
          | DWM_TNP_SOURCECLIENTAREAONLY,
        rcSource: rc_src,
        rcDestination: rc_dst,
        opacity: 255,
        fVisible: true.into(),
        fSourceClientAreaOnly: false.into(),
        ..Default::default()
      };
      // SAFETY: `thumbnail` is a valid handle. `props` is stack-allocated
      // and remains live for the duration of this call.
      unsafe {
        let _ = DwmUpdateThumbnailProperties(thumbnail, &raw const props);
      }
    }

    // Position the surrogate window at the constrained (monitor-clamped) rect.
    let constrained_rect = if is_vertical {
      Rect::from_xy(perp_pos, vis_start, perp_size, constrained)
    } else {
      Rect::from_xy(vis_start, perp_pos, constrained, perp_size)
    };
    let _ = inner.update(&constrained_rect, 255);

    if !self.is_visible {
      inner.set_visible(true);
      self.is_visible = true;
    }
  }
}
