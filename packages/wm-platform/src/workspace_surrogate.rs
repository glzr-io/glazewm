use windows::Win32::Foundation::{HWND, RECT};

use crate::{CornerStyle, NativeSurrogate, Rect};

/// Surrogate overlay for a single window participating in a workspace-switch
/// animation.
///
/// Both outgoing and incoming windows move together so the whole workspace
/// slides as a single panel. The surrogate is created at the full monitor rect
/// (`viewport`) and never repositioned; all per-frame animation is expressed
/// via `rcSource`/`rcDestination` in `DwmUpdateThumbnailProperties`, avoiding
/// any per-frame `SetWindowPos` calls. The surrogate is hidden (via `SW_HIDE`)
/// when the visible area is empty.
pub struct WorkspaceSurrogate {
  inner: NativeSurrogate,
  /// Final screen rect of the window (target position for incoming, current
  /// screen rect for outgoing).
  pub rect: Rect,
  /// Monitor rect used as the surrogate window's fixed position and size.
  ///
  /// The surrogate is created at `viewport` and stays there for the entire
  /// animation. `rcDestination` coordinates are expressed relative to
  /// `viewport`'s top-left corner in every per-frame thumbnail update.
  viewport: Rect,
  /// DWM thumbnail opacity (0–255) derived from the window-effects config.
  opacity: u8,
  /// The non-full-opacity end of the opacity animation, as a fraction of
  /// `opacity` (0.0–1.0).
  ///
  /// For outgoing windows this is the final opacity fraction (start = 1.0,
  /// end = `opacity_endpoint`). For incoming windows it is the initial opacity
  /// fraction (start = `opacity_endpoint`, end = 1.0). At `1.0` (default) the
  /// opacity is constant throughout the animation; at `0.0` the window fully
  /// fades out or in.
  opacity_endpoint: f32,
}

impl WorkspaceSurrogate {
  /// Creates a hidden surrogate for a workspace-switch animation.
  ///
  /// `viewport` is the monitor rect; the surrogate window is fixed there for
  /// the entire animation. `rect` is the source window's screen rect, used as
  /// the thumbnail registration dimensions and the reference for per-frame
  /// coordinate math.
  ///
  /// `opacity_endpoint` controls how far the opacity animates away from the
  /// effect opacity. For outgoing windows pass `config.opacity_outgoing`; for
  /// incoming windows pass `config.opacity_incoming`. At `1.0` the opacity is
  /// held constant; at `0.0` the window fully fades.
  ///
  /// The surrogate is created hidden. For outgoing windows, call
  /// [`show_initial`] before cloaking the real window to avoid a blank frame.
  /// For incoming windows in stationary styles (fade/zoom), call
  /// [`show_incoming`] before the animation loop to pre-warm the DWM thumbnail.
  ///
  /// [`show_initial`]: WorkspaceSurrogate::show_initial
  /// [`show_incoming`]: WorkspaceSurrogate::show_incoming
  pub fn new(
    hwnd: HWND,
    rect: &Rect,
    viewport: &Rect,
    opacity: u8,
    opacity_endpoint: f32,
  ) -> crate::Result<Self> {
    let inner = NativeSurrogate::create(
      hwnd,
      viewport,
      rect,
      None,
      opacity,
      false,
      RECT::default(),
      // Workspace surrogates span the full viewport and must not be rounded.
      &CornerStyle::Square,
      // Workspace surrogates should sit just below the source window; they
      // don't compete with resize surrogates since workspace-switch and
      // resize animations are mutually exclusive.
      hwnd,
    )?;
    Ok(Self {
      inner,
      rect: rect.clone(),
      viewport: viewport.clone(),
      opacity,
      opacity_endpoint: opacity_endpoint.clamp(0.0, 1.0),
    })
  }

  /// Hides the DWM thumbnail without destroying it or hiding the surrogate window.
  ///
  /// Called immediately before the post-animation `DwmFlush` so the flush
  /// frame shows only the uncloaked real windows. Without this, DWM blends the
  /// thumbnail (at configured opacity) on top of the real window (also at
  /// configured opacity), producing a double-blend that appears fully opaque
  /// for one frame.
  pub fn hide_thumbnail(&mut self) {
    self.inner.set_thumbnail_visible(false);
  }

  /// Shows the surrogate at full opacity with the thumbnail at the window's
  /// natural (unscaled) position within the monitor viewport.
  ///
  /// Always uses opacity `255` (fully opaque) so the surrogate completely
  /// covers the real window before it is cloaked, avoiding a double-blend
  /// frame. Call [`apply_effect_opacity`] after the real window is cloaked to
  /// reduce the thumbnail to the configured `opacity`.
  ///
  /// [`apply_effect_opacity`]: WorkspaceSurrogate::apply_effect_opacity
  pub fn show_initial(&mut self) {
    let rc_src =
      RECT { left: 0, top: 0, right: self.rect.width(), bottom: self.rect.height() };
    let rc_dst = RECT {
      left: self.rect.left - self.viewport.left,
      top: self.rect.top - self.viewport.top,
      right: self.rect.right - self.viewport.left,
      bottom: self.rect.bottom - self.viewport.top,
    };
    self.inner.set_thumbnail_rects(rc_src, rc_dst);
    self.inner.set_window_opacity(u8::MAX);
    self.inner.set_visible(true);
  }

  /// Updates the DWM thumbnail opacity to the configured `opacity` without
  /// changing the surrogate window position or size.
  ///
  /// Call this after the real window has been cloaked so the thumbnail's
  /// effect opacity is applied without causing a double-blend with the
  /// real window underneath.
  pub fn apply_effect_opacity(&mut self) {
    self.inner.set_window_opacity(self.opacity);
  }

  /// Shows the surrogate at the incoming animation start opacity, with the
  /// thumbnail pre-positioned at the window's location within the monitor
  /// viewport.
  ///
  /// Use for incoming windows in stationary transitions (fade, zoom) so DWM
  /// warms the thumbnail before the animation begins. The start opacity is
  /// derived from `opacity_endpoint`: `opacity_endpoint * effect_opacity`. At
  /// `opacity_endpoint = 0.0` the surrogate is invisible but the thumbnail is
  /// registered; at `1.0` it starts fully opaque. [`update_fade`] or
  /// [`update_zoom`] then drives the per-frame opacity from this initial state.
  ///
  /// [`update_fade`]: WorkspaceSurrogate::update_fade
  /// [`update_zoom`]: WorkspaceSurrogate::update_zoom
  pub fn show_incoming(&mut self) {
    let rc_src =
      RECT { left: 0, top: 0, right: self.rect.width(), bottom: self.rect.height() };
    let rc_dst = RECT {
      left: self.rect.left - self.viewport.left,
      top: self.rect.top - self.viewport.top,
      right: self.rect.right - self.viewport.left,
      bottom: self.rect.bottom - self.viewport.top,
    };
    self.inner.set_thumbnail_rects(rc_src, rc_dst);
    self.inner.set_window_opacity(self.lerp_opacity(0.0, true));
    self.inner.set_visible(true);
  }

  /// Advances the surrogate opacity for a fade-only transition.
  ///
  /// The surrogate stays at its target rect; only the window opacity is lerped
  /// each frame to produce a crossfade without positional movement.
  pub fn update_fade(&mut self, eased_progress: f32, is_incoming: bool) {
    self.inner.set_window_opacity(self.lerp_opacity(eased_progress, is_incoming));
  }

  /// Advances the surrogate along the horizontal axis to `eased_progress`
  /// (0.0 → 1.0).
  ///
  /// The visible strip is clipped to `[monitor_x, monitor_x + monitor_width]`
  /// via `rcSource`/`rcDestination`; the surrogate window itself does not move.
  /// `slide_distance` is the effective travel distance (may be less than
  /// `monitor_width` to close the seam gap between the two workspace panels).
  pub fn update_slide_horizontal(
    &mut self,
    eased_progress: f32,
    is_incoming: bool,
    direction: i32,
    monitor_x: i32,
    monitor_width: i32,
    slide_distance: i32,
  ) {
    self.slide_axis(
      eased_progress,
      is_incoming,
      direction,
      monitor_x,
      monitor_width,
      slide_distance,
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
    slide_distance: i32,
  ) {
    self.slide_axis(
      eased_progress,
      is_incoming,
      direction,
      monitor_y,
      monitor_height,
      slide_distance,
      true,
    );
  }

  /// Advances the surrogate along the horizontal axis with a simultaneous
  /// whole-workspace scale to `eased_progress` (0.0 → 1.0).
  ///
  /// Each surrogate is positioned at the scaled screen coordinates of its
  /// window (scaling from the monitor center), so the entire workspace
  /// shrinks/grows as one unit. The outgoing workspace scales from `1.0` to
  /// `1.0 - zoom_factor`; the incoming scales from `1.0 - zoom_factor` to
  /// `1.0`. `slide_distance` controls horizontal travel (see
  /// [`update_slide_horizontal`]).
  ///
  /// [`update_slide_horizontal`]: WorkspaceSurrogate::update_slide_horizontal
  pub fn update_slide_zoom_horizontal(
    &mut self,
    eased_progress: f32,
    is_incoming: bool,
    direction: i32,
    monitor_x: i32,
    monitor_width: i32,
    monitor_y: i32,
    monitor_height: i32,
    slide_distance: i32,
    zoom_factor: f32,
  ) {
    self.slide_zoom_axis(
      eased_progress,
      is_incoming,
      direction,
      monitor_x,
      monitor_width,
      monitor_y,
      monitor_height,
      slide_distance,
      zoom_factor,
      false,
    );
  }

  /// Advances the surrogate along the vertical axis with a simultaneous
  /// whole-workspace scale to `eased_progress` (0.0 → 1.0).
  ///
  /// Mirrors [`update_slide_zoom_horizontal`] on the y-axis.
  ///
  /// [`update_slide_zoom_horizontal`]: WorkspaceSurrogate::update_slide_zoom_horizontal
  pub fn update_slide_zoom_vertical(
    &mut self,
    eased_progress: f32,
    is_incoming: bool,
    direction: i32,
    monitor_x: i32,
    monitor_width: i32,
    monitor_y: i32,
    monitor_height: i32,
    slide_distance: i32,
    zoom_factor: f32,
  ) {
    self.slide_zoom_axis(
      eased_progress,
      is_incoming,
      direction,
      monitor_x,
      monitor_width,
      monitor_y,
      monitor_height,
      slide_distance,
      zoom_factor,
      true,
    );
  }

  /// Animates a zoom-from-center transition to `eased_progress` (0.0 → 1.0).
  ///
  /// Each surrogate independently zooms in (incoming) or out (outgoing) from
  /// its own center. `rcDestination` grows from a zero-size rect at the
  /// surrogate center to the full surrogate rect, scaling the source content
  /// via DWM. Opacity is lerped according to `opacity_endpoint`.
  pub fn update_zoom(&mut self, eased_progress: f32, is_incoming: bool) {
    let t = if is_incoming {
      eased_progress
    } else {
      1.0 - eased_progress
    };

    let w = self.rect.width();
    let h = self.rect.height();
    let half_w = (w as f32 / 2.0 * t).round() as i32;
    let half_h = (h as f32 / 2.0 * t).round() as i32;

    if half_w <= 0 || half_h <= 0 {
      self.inner.set_visible(false);
      return;
    }

    let cx = w / 2;
    let cy = h / 2;

    let rc_src = windows::Win32::Foundation::RECT {
      left: 0,
      top: 0,
      right: w,
      bottom: h,
    };
    let rc_dst = windows::Win32::Foundation::RECT {
      left: cx - half_w,
      top: cy - half_h,
      right: cx + half_w,
      bottom: cy + half_h,
    };

    self.inner.set_thumbnail_rects(rc_src, rc_dst);
    self.inner.set_window_opacity(self.lerp_opacity(eased_progress, is_incoming));
    self.inner.set_visible(true);
  }

  /// Computes the per-frame window opacity for a surrogate at `progress`
  /// (0.0 → 1.0).
  ///
  /// Outgoing: lerps from `opacity` → `opacity_endpoint * opacity`.
  /// Incoming: lerps from `opacity_endpoint * opacity` → `opacity`.
  /// When `opacity_endpoint` is `1.0` (default), the result is constant
  /// `opacity` — no fade.
  fn lerp_opacity(&self, progress: f32, is_incoming: bool) -> u8 {
    let (start_frac, end_frac): (f32, f32) = if is_incoming {
      (self.opacity_endpoint, 1.0)
    } else {
      (1.0, self.opacity_endpoint)
    };
    let frac = start_frac + (end_frac - start_frac) * progress;
    (self.opacity as f32 * frac.clamp(0.0, 1.0)).round() as u8
  }

  /// Shared implementation for [`update_slide_zoom_horizontal`] and
  /// [`update_slide_zoom_vertical`].
  ///
  /// Computes a per-frame scale from the monitor center (both axes) combined
  /// with a slide offset on the primary axis (`is_vertical` selects which).
  /// The surrogate is pinned at `self.viewport` (the monitor rect) for the
  /// entire animation; zoom and slide are expressed entirely via
  /// `rcSource`/`rcDestination` in `DwmUpdateThumbnailProperties`.
  ///
  /// [`update_slide_zoom_horizontal`]: WorkspaceSurrogate::update_slide_zoom_horizontal
  /// [`update_slide_zoom_vertical`]: WorkspaceSurrogate::update_slide_zoom_vertical
  #[allow(clippy::too_many_arguments)]
  fn slide_zoom_axis(
    &mut self,
    eased_progress: f32,
    is_incoming: bool,
    direction: i32,
    monitor_x: i32,
    monitor_width: i32,
    monitor_y: i32,
    monitor_height: i32,
    slide_distance: i32,
    zoom_factor: f32,
    is_vertical: bool,
  ) {
    // Outgoing: scale 1.0 → (1 - zoom_factor) as it exits.
    // Incoming: scale (1 - zoom_factor) → 1.0 as it enters.
    let zoom_t = if is_incoming {
      1.0 - eased_progress
    } else {
      eased_progress
    };
    let scale = 1.0 - zoom_factor * zoom_t;

    if scale <= 0.0 {
      self.inner.set_visible(false);
      return;
    }

    // Slide offset on the primary axis.
    let slide_offset = if is_incoming {
      (direction as f32 * slide_distance as f32 * (1.0 - eased_progress)) as i32
    } else {
      (-direction as f32 * slide_distance as f32 * eased_progress) as i32
    };

    // Zoom all four edges from the monitor center.
    let cx = monitor_x + monitor_width / 2;
    let cy = monitor_y + monitor_height / 2;
    let zoomed_left =
      cx + ((self.rect.left - cx) as f32 * scale).round() as i32;
    let zoomed_top =
      cy + ((self.rect.top - cy) as f32 * scale).round() as i32;
    let zoomed_right =
      cx + ((self.rect.right - cx) as f32 * scale).round() as i32;
    let zoomed_bottom =
      cy + ((self.rect.bottom - cy) as f32 * scale).round() as i32;

    // Apply slide offset on the primary axis only.
    let (final_left, final_top, final_right, final_bottom) = if is_vertical {
      (
        zoomed_left,
        zoomed_top + slide_offset,
        zoomed_right,
        zoomed_bottom + slide_offset,
      )
    } else {
      (
        zoomed_left + slide_offset,
        zoomed_top,
        zoomed_right + slide_offset,
        zoomed_bottom,
      )
    };

    // Clip to monitor bounds.
    let monitor_right = monitor_x + monitor_width;
    let monitor_bottom = monitor_y + monitor_height;
    let vis_left = final_left.max(monitor_x);
    let vis_top = final_top.max(monitor_y);
    let vis_right = final_right.min(monitor_right);
    let vis_bottom = final_bottom.min(monitor_bottom);

    if vis_left >= vis_right || vis_top >= vis_bottom {
      self.inner.set_visible(false);
      return;
    }

    // Map the visible screen area back to source-window coordinates.
    // screen_x = final_left + src_x * scale  →  src_x = (screen_x - final_left) / scale
    let ww = self.rect.right - self.rect.left;
    let wh = self.rect.bottom - self.rect.top;
    let src_left =
      (((vis_left - final_left) as f32 / scale).round() as i32).clamp(0, ww);
    let src_top =
      (((vis_top - final_top) as f32 / scale).round() as i32).clamp(0, wh);
    let src_right =
      (((vis_right - final_left) as f32 / scale).round() as i32).clamp(0, ww);
    let src_bottom =
      (((vis_bottom - final_top) as f32 / scale).round() as i32).clamp(0, wh);

    let rc_src =
      RECT { left: src_left, top: src_top, right: src_right, bottom: src_bottom };
    // `rcDestination` is relative to the surrogate's top-left, which is
    // pinned at `self.viewport` (monitor top-left) for the entire animation.
    let rc_dst = RECT {
      left: vis_left - self.viewport.left,
      top: vis_top - self.viewport.top,
      right: vis_right - self.viewport.left,
      bottom: vis_bottom - self.viewport.top,
    };
    self.inner.set_thumbnail_rects(rc_src, rc_dst);
    self.inner.set_window_opacity(self.lerp_opacity(eased_progress, is_incoming));
    self.inner.set_visible(true);
  }

  /// Shared implementation for [`update_slide_horizontal`] and
  /// [`update_slide_vertical`].
  ///
  /// The surrogate is pinned at `self.viewport` (the monitor rect) for the
  /// entire animation. The visible strip of source content is mapped to the
  /// corresponding screen area via `rcSource`/`rcDestination`, with no
  /// `SetWindowPos` calls per frame.
  ///
  /// [`update_slide_horizontal`]: WorkspaceSurrogate::update_slide_horizontal
  /// [`update_slide_vertical`]: WorkspaceSurrogate::update_slide_vertical
  fn slide_axis(
    &mut self,
    eased_progress: f32,
    is_incoming: bool,
    direction: i32,
    monitor_origin: i32,
    monitor_size: i32,
    slide_distance: i32,
    is_vertical: bool,
  ) {
    // Incoming: start at +direction*slide_distance offset, end at 0.
    // Outgoing: start at 0, end at -direction*slide_distance offset.
    let offset = if is_incoming {
      (direction as f32 * slide_distance as f32 * (1.0 - eased_progress)) as i32
    } else {
      (-direction as f32 * slide_distance as f32 * eased_progress) as i32
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
      self.inner.set_visible(false);
      return;
    }

    // Source-window-local start of the visible strip.
    let src_start = vis_start - current;
    let constrained = vis_end - vis_start;

    // `rcSource` is the visible slice of the source window.
    // `rcDestination` places that slice in screen space relative to the
    // surrogate's top-left (which equals `self.viewport`, the monitor rect).
    let (rc_src, rc_dst) = if is_vertical {
      (
        RECT {
          left: 0,
          top: src_start,
          right: perp_size,
          bottom: src_start + constrained,
        },
        RECT {
          left: perp_pos - self.viewport.left,
          top: vis_start - self.viewport.top,
          right: perp_pos + perp_size - self.viewport.left,
          bottom: vis_end - self.viewport.top,
        },
      )
    } else {
      (
        RECT {
          left: src_start,
          top: 0,
          right: src_start + constrained,
          bottom: perp_size,
        },
        RECT {
          left: vis_start - self.viewport.left,
          top: perp_pos - self.viewport.top,
          right: vis_end - self.viewport.left,
          bottom: perp_pos + perp_size - self.viewport.top,
        },
      )
    };
    self.inner.set_thumbnail_rects(rc_src, rc_dst);
    self.inner.set_window_opacity(self.lerp_opacity(eased_progress, is_incoming));
    self.inner.set_visible(true);
  }
}
