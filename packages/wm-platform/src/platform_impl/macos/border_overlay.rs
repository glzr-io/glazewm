use std::{
  collections::HashMap,
  ffi::c_void,
  sync::{LazyLock, Mutex},
};

use foreign_types::ForeignType;
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

  /// Releases the reference returned by `SLWindowContextCreate`.
  fn CGContextRelease(ctx: *mut c_void);
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

/// Serializes position changes without dispatching to the AX thread.
pub(crate) fn update_border_position(
  window_id: WindowId,
  frame: &Rect,
) -> crate::Result<()> {
  BORDER_OVERLAY_MANAGER
    .lock()
    .map_err(|_| {
      crate::Error::Platform(
        "Border overlay manager lock poisoned.".to_string(),
      )
    })?
    .update_position(window_id, frame)
}

/// Removes an overlay even when another border update is in flight.
pub(crate) fn remove_border(window_id: WindowId) -> crate::Result<()> {
  BORDER_OVERLAY_MANAGER
    .lock()
    .map_err(|_| {
      crate::Error::Platform(
        "Border overlay manager lock poisoned.".to_string(),
      )
    })?
    .remove(window_id);
  Ok(())
}

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

    if self.connection == 0 {
      return Ok(());
    }

    if frame.width() <= 0 || frame.height() <= 0 {
      self.remove(window_id);
      return Ok(());
    }

    if let Some(overlay) = self.overlays.get_mut(&window_id) {
      overlay.update(frame, color)?;
      overlay.backend.order(window_id.0)?;
      return Ok(());
    }

    let mut overlay = BorderOverlay {
      backend: SkyLightOverlay::new(self.connection, frame)?,
      frame: None,
      color: None,
    };
    overlay.update(frame, color)?;
    overlay.backend.order(window_id.0)?;
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
      if frame.width() <= 0 || frame.height() <= 0 {
        self.remove(window_id);
      } else if let Some(color) = overlay.color.clone() {
        overlay.update(frame, &color)?;
      }
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
struct BorderOverlay<B = SkyLightOverlay> {
  backend: B,
  frame: Option<Rect>,
  color: Option<Color>,
}

/// Native operations needed to apply a border's desired state.
trait OverlayBackend {
  fn reshape(&mut self, frame: &Rect) -> crate::Result<()>;
  fn move_to(&mut self, frame: &Rect) -> crate::Result<()>;
  fn redraw(&mut self, frame: &Rect, color: &Color) -> crate::Result<()>;
}

impl<B: OverlayBackend> BorderOverlay<B> {
  /// Invalidates cached geometry after failure because native operations
  /// may have partially applied. Retain the last successful color for
  /// subsequent position updates.
  fn update(&mut self, frame: &Rect, color: &Color) -> crate::Result<()> {
    let result = self.apply_update(frame, color);
    if result.is_err() {
      self.frame = None;
    }
    result
  }

  fn apply_update(
    &mut self,
    frame: &Rect,
    color: &Color,
  ) -> crate::Result<()> {
    let resized = self.frame.as_ref().is_none_or(|old| {
      old.width() != frame.width() || old.height() != frame.height()
    });
    if resized {
      self.backend.reshape(frame)?;
    } else if self.frame.as_ref() != Some(frame) {
      self.backend.move_to(frame)?;
    }
    if resized || self.color.as_ref() != Some(color) {
      self.backend.redraw(frame, color)?;
    }
    self.frame = Some(frame.clone());
    self.color = Some(color.clone());
    Ok(())
  }
}

/// Owns the native window and its drawing context, including while
/// construction is incomplete.
struct SkyLightOverlay {
  wid: SLSWindow,
  connection: SLSConnection,
  context: Option<core_graphics::context::CGContext>,
}

// SAFETY: All context and window access is serialized by
// `BORDER_OVERLAY_MANAGER`. No drawing context escapes that lock.
unsafe impl Send for SkyLightOverlay {}

impl SkyLightOverlay {
  /// Creates a native overlay with cleanup active before any fallible
  /// configuration or context allocation.
  fn new(connection: SLSConnection, frame: &Rect) -> crate::Result<Self> {
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

    let mut overlay = Self {
      wid,
      connection,
      context: None,
    };

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

    // SAFETY: The live overlay owns the returned context reference.
    let ctx_ref = unsafe {
      ffi::SLWindowContextCreate(connection, wid, std::ptr::null())
    };
    if ctx_ref.is_null() {
      return Err(crate::Error::Platform(
        "SLWindowContextCreate returned null context.".to_string(),
      ));
    }
    // SAFETY: This wrapper retains the valid context. Release the
    // original creation reference after adopting the retained wrapper.
    overlay.context = Some(unsafe {
      core_graphics::context::CGContext::from_existing_context_ptr(
        ctx_ref.cast(),
      )
    });
    unsafe { CGContextRelease(ctx_ref) };
    Ok(overlay)
  }

  /// Restores the overlay's ordering relative to its target on focus.
  fn order(&self, target_wid: SLSWindow) -> crate::Result<()> {
    // SAFETY: Both IDs identify live windows on this connection.
    ensure_sls_success("SLSOrderWindow", unsafe {
      SLSOrderWindow(self.connection, self.wid, 1, target_wid)
    })
  }
}

impl OverlayBackend for SkyLightOverlay {
  fn reshape(&mut self, frame: &Rect) -> crate::Result<()> {
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

    Ok(())
  }

  fn move_to(&mut self, frame: &Rect) -> crate::Result<()> {
    let origin = CGPoint::new(
      f64::from(frame.x()) - BORDER_WIDTH,
      f64::from(frame.y()) - BORDER_WIDTH,
    );
    // SAFETY: The overlay is live and origin remains valid for the call.
    ensure_sls_success("SLSMoveWindow", unsafe {
      ffi::SLSMoveWindow(self.connection, self.wid, &raw const origin)
    })
  }

  fn redraw(&mut self, frame: &Rect, color: &Color) -> crate::Result<()> {
    let ctx = self.context.as_ref().ok_or_else(|| {
      crate::Error::Platform("Missing overlay context.".to_string())
    })?;

    let outset = BORDER_WIDTH;
    let overlay_w = f64::from(frame.width()) + outset * 2.0;
    let overlay_h = f64::from(frame.height()) + outset * 2.0;

    ctx.clear_rect(core_graphics::geometry::CGRect::new(
      &core_graphics::geometry::CGPoint::new(0.0, 0.0),
      &core_graphics::geometry::CGSize::new(overlay_w, overlay_h),
    ));

    ctx.set_rgb_stroke_color(
      f64::from(color.r) / 255.0,
      f64::from(color.g) / 255.0,
      f64::from(color.b) / 255.0,
      f64::from(color.a) / 255.0,
    );
    ctx.set_line_width(BORDER_WIDTH);

    let half = BORDER_WIDTH / 2.0;
    let path_rect = core_graphics::geometry::CGRect::new(
      &core_graphics::geometry::CGPoint::new(outset - half, outset - half),
      &core_graphics::geometry::CGSize::new(
        f64::from(frame.width()) + BORDER_WIDTH,
        f64::from(frame.height()) + BORDER_WIDTH,
      ),
    );

    let max_radius = path_rect.size.width.min(path_rect.size.height) / 2.0;
    let r = BORDER_RADIUS.min(max_radius);

    // SAFETY: `CGPathCreateWithRoundedRect` returns a retained path.
    // Released after stroking.
    let path = unsafe {
      CGPathCreateWithRoundedRect(path_rect, r, r, std::ptr::null())
    };

    if path.is_null() {
      return Err(crate::Error::Platform(
        "Failed to create border path.".to_string(),
      ));
    }
    {
      // SAFETY: Both context and retained path are valid.
      unsafe { CGContextAddPath(ctx.as_ptr().cast(), path) };
      ctx.stroke_path();

      // SAFETY: Releasing the retained path from
      // `CGPathCreateWithRoundedRect`.
      unsafe { CGPathRelease(path) };
    }

    ctx.flush();
    // SAFETY: The overlay is live; null requests the whole content region.
    ensure_sls_success("SLSFlushWindowContentRegion", unsafe {
      ffi::SLSFlushWindowContentRegion(
        self.connection,
        self.wid,
        std::ptr::null(),
      )
    })
  }
}

impl Drop for SkyLightOverlay {
  fn drop(&mut self) {
    // Release the context before its backing window.
    self.context.take();
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

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Default)]
  struct RecordingBackend {
    calls: Vec<&'static str>,
    fail: Option<&'static str>,
  }

  impl RecordingBackend {
    fn record(&mut self, operation: &'static str) -> crate::Result<()> {
      self.calls.push(operation);
      if self.fail == Some(operation) {
        return Err(crate::Error::Platform("Injected failure".into()));
      }
      Ok(())
    }
  }

  impl OverlayBackend for RecordingBackend {
    fn reshape(&mut self, _: &Rect) -> crate::Result<()> {
      self.record("reshape")
    }
    fn move_to(&mut self, _: &Rect) -> crate::Result<()> {
      self.record("move")
    }
    fn redraw(&mut self, _: &Rect, _: &Color) -> crate::Result<()> {
      self.record("draw")
    }
  }

  fn overlay() -> BorderOverlay<RecordingBackend> {
    BorderOverlay {
      backend: RecordingBackend::default(),
      frame: None,
      color: None,
    }
  }

  fn color() -> Color {
    Color {
      r: 255,
      g: 100,
      b: 0,
      a: 255,
    }
  }

  #[test]
  fn creation_draws_once_and_identical_updates_do_nothing() {
    let mut overlay = overlay();
    let frame = Rect::from_xy(10, 20, 800, 600);
    overlay.update(&frame, &color()).unwrap();
    assert_eq!(overlay.backend.calls, ["reshape", "draw"]);
    overlay.backend.calls.clear();
    overlay.update(&frame, &color()).unwrap();
    assert!(overlay.backend.calls.is_empty());
  }

  #[test]
  fn repeated_drag_positions_and_final_drop_only_move() {
    let mut overlay = overlay();
    overlay
      .update(&Rect::from_xy(0, 0, 800, 600), &color())
      .unwrap();
    overlay.backend.calls.clear();
    for x in [10, 20, 30] {
      overlay
        .update(&Rect::from_xy(x, 0, 800, 600), &color())
        .unwrap();
    }
    assert_eq!(overlay.backend.calls, ["move", "move", "move"]);
    assert_eq!(overlay.frame, Some(Rect::from_xy(30, 0, 800, 600)));
  }

  #[test]
  fn color_and_combined_resize_updates_draw_once() {
    let mut overlay = overlay();
    let frame = Rect::from_xy(10, 20, 800, 600);
    overlay.update(&frame, &color()).unwrap();
    overlay.backend.calls.clear();
    let changed = Color { a: 128, ..color() };
    overlay.update(&frame, &changed).unwrap();
    assert_eq!(overlay.backend.calls, ["draw"]);
    overlay.backend.calls.clear();
    overlay
      .update(&Rect::from_xy(20, 30, 900, 700), &color())
      .unwrap();
    assert_eq!(overlay.backend.calls, ["reshape", "draw"]);
  }

  #[test]
  fn failed_native_operations_do_not_commit_and_identical_requests_retry()
  {
    for (operation, requested, expected) in [
      (
        "move",
        Rect::from_xy(20, 30, 800, 600),
        vec!["reshape", "draw"],
      ),
      (
        "reshape",
        Rect::from_xy(20, 30, 900, 700),
        vec!["reshape", "draw"],
      ),
      (
        "draw",
        Rect::from_xy(10, 20, 800, 600),
        vec!["reshape", "draw"],
      ),
    ] {
      let mut overlay = overlay();
      let original = Rect::from_xy(10, 20, 800, 600);
      overlay.update(&original, &color()).unwrap();
      let changed = Color { r: 0, ..color() };
      overlay.backend.fail = Some(operation);
      assert!(overlay.update(&requested, &changed).is_err());
      assert_eq!(overlay.frame, None);
      assert_eq!(overlay.color, Some(color()));
      overlay.backend.fail = None;
      overlay.backend.calls.clear();
      overlay.update(&requested, &changed).unwrap();
      assert_eq!(overlay.backend.calls, expected);
      assert_eq!(overlay.frame, Some(requested));
      assert_eq!(overlay.color, Some(changed));
    }
  }

  #[test]
  fn partial_resize_failure_can_restore_the_previous_frame() {
    let mut overlay = overlay();
    let original = Rect::from_xy(10, 20, 800, 600);
    overlay.update(&original, &color()).unwrap();
    overlay.backend.fail = Some("draw");
    assert!(overlay
      .update(&Rect::from_xy(10, 20, 900, 700), &color())
      .is_err());
    overlay.backend.fail = None;
    overlay.backend.calls.clear();
    overlay.update(&original, &color()).unwrap();
    assert_eq!(overlay.backend.calls, ["reshape", "draw"]);
  }

  #[test]
  fn disabled_manager_never_creates_native_overlays() {
    let mut manager = BorderOverlayManager::disabled();
    manager
      .set_border_color(
        WindowId(123),
        &Rect::from_xy(0, 0, 800, 600),
        Some(&color()),
      )
      .unwrap();
    assert!(manager.overlays.is_empty());
  }

  #[test]
  fn native_overlay_smoke() {
    // The custom main-thread test harness does not support #[ignore].
    // Opt in only in a logged-in WindowServer session.
    if std::env::var_os("GLAZEWM_NATIVE_OVERLAY_TEST").is_none() {
      return;
    }
    let connection = unsafe { ffi::SLSMainConnectionID() };
    assert_ne!(connection, 0);
    // Never order this isolated off-screen overlay into the user's
    // desktop.
    let frame = Rect::from_xy(-9999, -9999, 800, 600);
    let mut overlay = BorderOverlay {
      backend: SkyLightOverlay::new(connection, &frame).unwrap(),
      frame: None,
      color: None,
    };
    overlay.update(&frame, &color()).unwrap();
    overlay
      .update(&Rect::from_xy(-9900, -9900, 800, 600), &color())
      .unwrap();
    let mut moved_bounds = objc2_core_foundation::CGRect::default();
    let status = unsafe {
      ffi::SLSGetWindowBounds(
        connection,
        overlay.backend.wid,
        &raw mut moved_bounds,
      )
    };
    assert_eq!(status, objc2_core_graphics::CGError::Success);
    assert_eq!(moved_bounds.origin, CGPoint::new(-9901.0, -9901.0));
    assert_eq!(moved_bounds.size, CGSize::new(802.0, 602.0));
    overlay
      .update(
        &Rect::from_xy(-9900, -9900, 900, 700),
        &Color { b: 255, ..color() },
      )
      .unwrap();
    let mut bounds = objc2_core_foundation::CGRect::default();
    let status = unsafe {
      ffi::SLSGetWindowBounds(
        connection,
        overlay.backend.wid,
        &raw mut bounds,
      )
    };
    assert_eq!(status, objc2_core_graphics::CGError::Success);
    assert_eq!(bounds.origin, CGPoint::new(-9901.0, -9901.0));
    assert_eq!(bounds.size, CGSize::new(902.0, 702.0));
  }
}
