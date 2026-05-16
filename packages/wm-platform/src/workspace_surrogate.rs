use windows::Win32::Foundation::{HWND, RECT};

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
  inner: NativeSurrogate,
  /// Final screen rect of the window (target position for incoming, current
  /// screen rect for outgoing).
  pub rect: Rect,
  /// DWM thumbnail opacity (0–255) derived from the window-effects config.
  opacity: u8,
  /// When `true`, the surrogate's opacity is lerped each frame: outgoing
  /// fades out (opacity → 0) and incoming fades in (0 → opacity).
  fade: bool,
}

impl WorkspaceSurrogate {
  /// Creates a hidden surrogate for a workspace-switch slide animation.
  ///
  /// The surrogate is created hidden. For outgoing windows, call
  /// [`show_initial`] before cloaking the real window to avoid a blank frame.
  /// For incoming windows, [`slide_axis`] reveals the surrogate as soon as it
  /// enters the monitor's visible area.
  ///
  /// When `fade` is `true`, [`slide_axis`] lerps the surrogate opacity each
  /// frame: outgoing fades from `opacity` → 0; incoming fades from 0 →
  /// `opacity`. [`apply_effect_opacity`] must still be called after cloaking
  /// the outgoing real window to set the animation-start opacity correctly.
  ///
  /// [`show_initial`]: WorkspaceSurrogate::show_initial
  /// [`slide_axis`]: WorkspaceSurrogate::slide_axis
  /// [`apply_effect_opacity`]: WorkspaceSurrogate::apply_effect_opacity
  pub fn new(
    hwnd: HWND,
    rect: &Rect,
    color: Option<&Color>,
    opacity: u8,
    fade: bool,
  ) -> crate::Result<Self> {
    let inner = NativeSurrogate::create(
      hwnd,
      rect,
      rect,
      color,
      opacity,
      false,
      RECT::default(),
      false,
    )?;
    Ok(Self { inner, rect: rect.clone(), opacity, fade })
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

  /// Shows the surrogate at full opacity covering its full rect.
  ///
  /// Always uses opacity `255` (fully opaque) so the surrogate completely
  /// covers the real window before it is cloaked, avoiding a double-blend
  /// frame. Call [`apply_effect_opacity`] after the real window is cloaked to
  /// reduce the thumbnail to the configured `opacity`.
  ///
  /// [`apply_effect_opacity`]: WorkspaceSurrogate::apply_effect_opacity
  pub fn show_initial(&mut self) {
    let _ = self.inner.update(&self.rect, u8::MAX);
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
    self.slide_axis(eased_progress, is_incoming, direction, monitor_x, monitor_width, false);
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
    self.slide_axis(eased_progress, is_incoming, direction, monitor_y, monitor_height, true);
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
      self.inner.set_visible(false);
      return;
    }

    let constrained = vis_end - vis_start;
    // Source-window-local start of the visible strip.
    let src_start = vis_start - current;

    // When a window exits the trailing edge of the monitor (`src_start == 0`
    // and the window still extends past `monitor_end`), both the surrogate
    // position and size would change every frame, creating a rendering
    // asymmetry. Fix: anchor the surrogate at `monitor_end - axis_size` and
    // shift only `rcDestination` inward so no `SetWindowPos` position change
    // occurs. The uncovered surrogate area is transparent (WS_POPUP default).
    let trailing_exit = src_start == 0 && current + axis_size > monitor_end;

    // Update the DWM thumbnail: show only the visible slice of source
    // content, mapped 1:1 onto the constrained surrogate rect.
    let (rc_src, rc_dst) = if is_vertical {
      if trailing_exit {
        let dest_top = axis_size - constrained;
        (
          RECT { left: 0, top: 0, right: perp_size, bottom: constrained },
          RECT { left: 0, top: dest_top, right: perp_size, bottom: axis_size },
        )
      } else {
        (
          RECT { left: 0, top: src_start, right: perp_size, bottom: src_start + constrained },
          RECT { left: 0, top: 0, right: perp_size, bottom: constrained },
        )
      }
    } else if trailing_exit {
      let dest_left = axis_size - constrained;
      (
        RECT { left: 0, top: 0, right: constrained, bottom: perp_size },
        RECT { left: dest_left, top: 0, right: axis_size, bottom: perp_size },
      )
    } else {
      (
        RECT { left: src_start, top: 0, right: src_start + constrained, bottom: perp_size },
        RECT { left: 0, top: 0, right: constrained, bottom: perp_size },
      )
    };
    self.inner.set_thumbnail_rects(rc_src, rc_dst);

    // For trailing-exit: fix the surrogate at `monitor_end - axis_size` so
    // only `rcDestination` changes each frame (no position movement).
    let (surr_pos, surr_size) = if trailing_exit {
      (monitor_end - axis_size, axis_size)
    } else {
      (vis_start, constrained)
    };

    let constrained_rect = if is_vertical {
      Rect::from_xy(perp_pos, surr_pos, perp_size, surr_size)
    } else {
      Rect::from_xy(surr_pos, perp_pos, surr_size, perp_size)
    };
    let _ = self.inner.reposition(&constrained_rect);

    if self.fade {
      let fade_alpha = if is_incoming {
        (self.opacity as f32 * eased_progress).round() as u8
      } else {
        (self.opacity as f32 * (1.0 - eased_progress)).round() as u8
      };
      self.inner.set_window_opacity(fade_alpha);
    }

    self.inner.set_visible(true);
  }
}
