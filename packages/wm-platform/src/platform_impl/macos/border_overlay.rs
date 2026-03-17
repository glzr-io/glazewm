use std::{
  collections::HashMap,
  ffi::c_void,
  sync::{LazyLock, Mutex},
};

use objc2_core_foundation::{CGPoint, CGSize};

use crate::{
  platform_impl::ffi::{
    self, CGSNewRegionWithRect, CGSReleaseRegion, SLSConnection,
    SLSOrderWindow, SLSReleaseWindow, SLSSetWindowOpacity,
    SLSSetWindowResolution, SLSSetWindowShape, SLSSetWindowTags,
    SLSWindow, SLSWindowSetShadowProperties,
  },
  Color, Rect, WindowId,
};

const BORDER_WIDTH: f64 = 1.0;
const BORDER_RADIUS: f64 = 17.0;

unsafe extern "C" {
  /// Creates a `CGPath` with rounded corners matching macOS native
  /// window corner curves.
  fn CGPathCreateWithRoundedRect(
    rect: core_graphics::geometry::CGRect,
    corner_width: f64,
    corner_height: f64,
    transform: *const core_graphics::geometry::CGAffineTransform,
  ) -> *const c_void;

  /// Releases a `CGPath`.
  fn CGPathRelease(path: *const c_void);

  /// Adds a `CGPath` to a `CGContext`.
  fn CGContextAddPath(ctx: *mut c_void, path: *const c_void);
}

/// Global manager for all border overlays on macOS.
pub(crate) static BORDER_OVERLAY_MANAGER: LazyLock<
  Mutex<BorderOverlayManager>,
> = LazyLock::new(|| {
  let manager = BorderOverlayManager::new().unwrap_or_else(|error| {
      tracing::error!(
        "Failed to initialize border overlay manager: {error}. Borders disabled."
      );
      BorderOverlayManager::disabled()
    });

  Mutex::new(manager)
});

/// Manages all overlay windows used for drawing borders.
pub(crate) struct BorderOverlayManager {
  overlays: HashMap<WindowId, BorderOverlay>,
  connection: SLSConnection,
}

impl BorderOverlayManager {
  /// Creates a new `BorderOverlayManager`.
  pub(crate) fn new() -> crate::Result<Self> {
    // SAFETY: SkyLight API returns the process main connection ID.
    let connection = unsafe { ffi::SLSMainConnectionID() };

    if connection == 0 {
      return Err(crate::Error::Platform(
        "SLS: SLSMainConnectionID returned invalid connection 0"
          .to_string(),
      ));
    }

    Ok(Self {
      overlays: HashMap::new(),
      connection,
    })
  }

  /// Sets, updates, or removes border color for a target window.
  pub(crate) fn set_border_color(
    &mut self,
    window_id: WindowId,
    frame: &Rect,
    color: Option<&Color>,
  ) -> crate::Result<()> {
    let Some(color) = color else {
      self.remove(window_id);
      return Ok(());
    };

    if frame.width() == 0 || frame.height() == 0 {
      self.remove(window_id);
      return Ok(());
    }

    if let Some(overlay) = self.overlays.get_mut(&window_id) {
      overlay.update_frame(frame)?;
      overlay.update_color(color);
      return Ok(());
    }

    let overlay =
      BorderOverlay::new(self.connection, window_id.0, frame, color)?;
    self.overlays.insert(window_id, overlay);
    Ok(())
  }

  /// Updates overlay position and size for a target window.
  pub(crate) fn update_position(
    &mut self,
    window_id: WindowId,
    frame: &Rect,
  ) -> crate::Result<()> {
    if let Some(overlay) = self.overlays.get_mut(&window_id) {
      overlay.update_frame(frame)?;
    }

    Ok(())
  }

  /// Removes and drops the overlay for a target window.
  pub(crate) fn remove(&mut self, window_id: WindowId) {
    self.overlays.remove(&window_id);
  }

  /// Creates a no-op manager used when initialization fails.
  fn disabled() -> Self {
    Self {
      overlays: HashMap::new(),
      connection: 0,
    }
  }
}

/// Overlay window that draws border for one target window.
struct BorderOverlay {
  wid: SLSWindow,
  connection: SLSConnection,
  frame: Rect,
  color: Color,
}

impl BorderOverlay {
  /// Creates a new `BorderOverlay` window for the target window.
  fn new(
    connection: SLSConnection,
    target_wid: SLSWindow,
    frame: &Rect,
    color: &Color,
  ) -> crate::Result<Self> {
    // Create a region for the initial window frame (required by
    // `SLSNewWindow`).
    let init_rect = objc2_core_foundation::CGRect::new(
      CGPoint::new(0.0, 0.0),
      CGSize::new(f64::from(frame.width()), f64::from(frame.height())),
    );

    let mut region: *const c_void = std::ptr::null();
    // SAFETY: `region` is a valid out-pointer, `init_rect` lives for
    // the call duration.
    let status = unsafe {
      CGSNewRegionWithRect(&raw const init_rect, &raw mut region)
    };
    ensure_sls_success("CGSNewRegionWithRect", status)?;

    let mut wid = 0;
    // SAFETY: `connection` is from SkyLight, `region` is valid from
    // `CGSNewRegionWithRect`, `wid` is a valid out-pointer.
    let status = unsafe {
      ffi::SLSNewWindow(
        connection,
        2,
        -9999.0_f32,
        -9999.0_f32,
        region,
        &raw mut wid,
      )
    };

    // SAFETY: Region was created by `CGSNewRegionWithRect` and must
    // be released.
    unsafe { CGSReleaseRegion(region) };
    ensure_sls_success("SLSNewWindow", status)?;

    // SAFETY: `wid` is a live SkyLight window created above.
    let status = unsafe { SLSSetWindowOpacity(connection, wid, false) };
    ensure_sls_success("SLSSetWindowOpacity", status)?;

    let set_tags: u64 = (1 << 1) | (1 << 9);
    let clear_tags: u64 = 0;
    // SAFETY: `wid` is a live SkyLight window, tag pointers are valid.
    // Bit 1 = floating, bit 9 = ignore mouse events (click-through).
    unsafe {
      SLSSetWindowTags(connection, wid, &raw const set_tags, 64);
      ffi::SLSClearWindowTags(connection, wid, &raw const clear_tags, 64);
    };

    // Disable shadow — non-fatal, skip on failure.
    let shadow_status =
      unsafe { SLSWindowSetShadowProperties(wid, std::ptr::null()) };
    if shadow_status != 0 {
      tracing::debug!(
        "SLSWindowSetShadowProperties returned {shadow_status} (non-fatal)."
      );
    }

    // SAFETY: `wid` is a live SkyLight window created above.
    let status = unsafe { SLSSetWindowResolution(connection, wid, 2.0) };
    ensure_sls_success("SLSSetWindowResolution", status)?;

    // SAFETY: Orders overlay above `target_wid`.
    let status = unsafe { SLSOrderWindow(connection, wid, 1, target_wid) };
    ensure_sls_success("SLSOrderWindow", status)?;

    let mut overlay = Self {
      wid,
      connection,
      frame: frame.clone(),
      color: color.clone(),
    };

    overlay.update_frame(frame)?;
    overlay.update_color(color);
    Ok(overlay)
  }

  /// Updates border color and redraws the overlay.
  fn update_color(&mut self, color: &Color) {
    self.color = color.clone();
    self.redraw();
  }

  /// Updates overlay frame (position + shape) and redraws the border.
  fn update_frame(&mut self, frame: &Rect) -> crate::Result<()> {
    self.frame = frame.clone();

    let outset = BORDER_WIDTH;
    let overlay_w = f64::from(frame.width()) + outset * 2.0;
    let overlay_h = f64::from(frame.height()) + outset * 2.0;
    let overlay_x = f64::from(frame.x()) - outset;
    let overlay_y = f64::from(frame.y()) - outset;

    let shape_rect = objc2_core_foundation::CGRect::new(
      CGPoint::new(0.0, 0.0),
      CGSize::new(overlay_w, overlay_h),
    );

    let mut region: *const c_void = std::ptr::null();
    // SAFETY: `region` is a valid out-pointer, `shape_rect` lives for
    // the call duration.
    let status = unsafe {
      CGSNewRegionWithRect(&raw const shape_rect, &raw mut region)
    };
    ensure_sls_success("CGSNewRegionWithRect", status)?;

    // SAFETY: `self.wid` and `self.connection` refer to a live
    // SkyLight window. x/y position the overlay on screen.
    #[allow(clippy::cast_possible_truncation)]
    let status = unsafe {
      SLSSetWindowShape(
        self.connection,
        self.wid,
        overlay_x as f32,
        overlay_y as f32,
        region,
      )
    };

    // SAFETY: Region was created by `CGSNewRegionWithRect`.
    unsafe { CGSReleaseRegion(region) };
    ensure_sls_success("SLSSetWindowShape", status)?;

    self.redraw();
    Ok(())
  }

  /// Redraws the border using the current color and frame.
  fn redraw(&mut self) {
    // SAFETY: Creates a drawing context owned by SkyLight for this
    // live window. `options` is null for default context.
    let ctx_ref = unsafe {
      ffi::SLWindowContextCreate(
        self.connection,
        self.wid,
        std::ptr::null(),
      )
    };

    if ctx_ref.is_null() {
      tracing::warn!("SLWindowContextCreate returned null context.");
      return;
    }

    // SAFETY: `ctx_ref` is a valid `CGContextRef` from SkyLight.
    let ctx = unsafe {
      core_graphics::context::CGContext::from_existing_context_ptr(
        ctx_ref.cast(),
      )
    };

    let outset = BORDER_WIDTH;
    let overlay_w = f64::from(self.frame.width()) + outset * 2.0;
    let overlay_h = f64::from(self.frame.height()) + outset * 2.0;

    ctx.clear_rect(core_graphics::geometry::CGRect::new(
      &core_graphics::geometry::CGPoint::new(0.0, 0.0),
      &core_graphics::geometry::CGSize::new(overlay_w, overlay_h),
    ));

    ctx.set_rgb_stroke_color(
      f64::from(self.color.r) / 255.0,
      f64::from(self.color.g) / 255.0,
      f64::from(self.color.b) / 255.0,
      f64::from(self.color.a) / 255.0,
    );
    ctx.set_line_width(BORDER_WIDTH);

    let half = BORDER_WIDTH / 2.0;
    let path_rect = core_graphics::geometry::CGRect::new(
      &core_graphics::geometry::CGPoint::new(outset - half, outset - half),
      &core_graphics::geometry::CGSize::new(
        f64::from(self.frame.width()) + BORDER_WIDTH,
        f64::from(self.frame.height()) + BORDER_WIDTH,
      ),
    );

    let max_radius = path_rect.size.width.min(path_rect.size.height) / 2.0;
    let r = BORDER_RADIUS.min(max_radius);

    // SAFETY: `CGPathCreateWithRoundedRect` returns a retained path.
    // Released after stroking.
    let path = unsafe {
      CGPathCreateWithRoundedRect(path_rect, r, r, std::ptr::null())
    };

    if !path.is_null() {
      // SAFETY: `ctx_ref` is a valid `CGContextRef`, `path` is a valid
      // retained `CGPathRef`. `CGContextAddPath` copies the path data.
      unsafe { CGContextAddPath(ctx_ref, path) };
      ctx.stroke_path();

      // SAFETY: Releasing the retained path from
      // `CGPathCreateWithRoundedRect`.
      unsafe { CGPathRelease(path) };
    }

    ctx.flush();
  }
}

impl Drop for BorderOverlay {
  fn drop(&mut self) {
    // SAFETY: Order the overlay window out (mode 0) before releasing.
    // `SLSReleaseWindow` alone only decrements the reference count and
    // may leave the window visible.
    unsafe {
      SLSOrderWindow(self.connection, self.wid, 0, 0);
      SLSReleaseWindow(self.connection, self.wid);
    }
  }
}

/// Returns `Ok(())` if the `SkyLight` status code is 0, otherwise an
/// error.
fn ensure_sls_success(
  function_name: &str,
  code: i32,
) -> crate::Result<()> {
  if code == 0 {
    return Ok(());
  }

  Err(crate::Error::Platform(format!(
    "SLS: {function_name} failed with code {code}"
  )))
}
