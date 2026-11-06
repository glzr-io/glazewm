use std::sync::{
  atomic::{AtomicU64, Ordering},
  Arc, Mutex,
};

use objc2::MainThreadMarker;
use objc2_app_kit::{
  NSApplicationActivationOptions, NSWindow, NSWorkspace,
};
use objc2_application_services::{AXError, AXValue};
use objc2_core_foundation::{
  CFBoolean, CFRetained, CFString, CGPoint, CGRect, CGSize,
};
use objc2_core_graphics::{CGDisplayIsAsleep, CGError};

use crate::{
  platform_impl::{
    self, ffi, AXUIElement, AXUIElementExt, AXValueExt, Application,
  },
  Dispatcher, FixedWindowSize, Point, Rect, ThreadBound, WindowId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FrameAttribute {
  Size,
  Position,
}

/// Writes only position for a same-size move. Resizing retains the
/// size-position-size workaround required by apps such as Firefox.
fn apply_frame(
  current: Option<&Rect>,
  requested: &Rect,
  force_size: bool,
  mut write: impl FnMut(FrameAttribute) -> crate::Result<()>,
) -> crate::Result<()> {
  if !force_size && current == Some(requested) {
    return Ok(());
  }
  let resize = force_size
    || current.is_none_or(|frame| {
      frame.width() != requested.width()
        || frame.height() != requested.height()
    });
  if resize {
    write(FrameAttribute::Size)?;
  }
  write(FrameAttribute::Position)?;
  if resize {
    write(FrameAttribute::Size)?;
  }
  Ok(())
}

/// AX can echo a refused request before the app commits its real size.
/// An inconclusive observation must not be cached as a completed probe.
fn ensure_size_settled(
  ax: (f64, f64),
  committed: (i32, i32),
) -> crate::Result<()> {
  if (ax.0 - f64::from(committed.0)).abs() > 0.5
    || (ax.1 - f64::from(committed.1)).abs() > 0.5
  {
    Err(crate::Error::Platform(
      "Window resize has not settled; retry fixed-size detection later."
        .into(),
    ))
  } else {
    Ok(())
  }
}

fn resolve_resizability(
  result: AXError,
  settable: bool,
) -> crate::Result<bool> {
  match result {
    AXError::Success => Ok(settable),
    // Missing capability metadata is not evidence of a fixed-size window.
    AXError::AttributeUnsupported | AXError::NotImplemented => Ok(true),
    _ => Err(crate::Error::Accessibility("AXSize".into(), result.0)),
  }
}

/// Containment must use the accepted native size, never the tile size.
fn contain_native_frame(
  bounds: &Rect,
  requested: Option<&Rect>,
  mut read: impl FnMut() -> crate::Result<Rect>,
  move_to: impl FnOnce(&Rect) -> crate::Result<()>,
) -> crate::Result<Rect> {
  let frame = read()?;
  let positioned = requested.map_or_else(
    || frame.clone(),
    |requested| {
      Rect::from_xy(
        requested.x(),
        requested.y(),
        frame.width(),
        frame.height(),
      )
    },
  );
  let contained = positioned.translate_within(bounds);
  if contained == frame {
    Ok(frame)
  } else {
    move_to(&contained)?;
    read()
  }
}

/// Some apps reject `AXSize` with an error rather than accepting a clamped
/// size. Still move their accepted frame onto the assigned monitor.
fn place_visible_frame(
  requested: &Rect,
  bounds: &Rect,
  apply: impl FnOnce() -> crate::Result<()>,
  read: impl FnMut() -> crate::Result<Rect>,
  mut move_to: impl FnMut(&Rect) -> crate::Result<()>,
) -> crate::Result<Rect> {
  if let Err(error) = apply() {
    tracing::debug!(
      "Frame write failed; positioning the accepted native size: {error}"
    );
    move_to(requested)?;
  }
  contain_native_frame(bounds, Some(requested), read, move_to)
}

/// Runs a queued write only if no newer command or user drag replaced it.
fn apply_frame_at_revision(
  current: &AtomicU64,
  expected: u64,
  write: impl FnOnce() -> crate::Result<()>,
) -> crate::Result<()> {
  if current.load(Ordering::SeqCst) == expected {
    write()?;
  }
  Ok(())
}

/// Platform-specific implementation of [`NativeWindow`].
#[derive(Clone, Debug)]
pub(crate) struct NativeWindow {
  pub(crate) id: WindowId,
  pub(crate) element: Arc<ThreadBound<CFRetained<AXUIElement>>>,
  pub(crate) application: Application,
  frame_revision: Arc<AtomicU64>,
  commanded_frames: Arc<Mutex<Option<[Rect; 2]>>>,
}

impl NativeWindow {
  /// Creates an instance of `NativeWindow`.
  #[must_use]
  pub(crate) fn new(
    id: WindowId,
    element: ThreadBound<CFRetained<AXUIElement>>,
    application: Application,
  ) -> Self {
    Self {
      element: Arc::new(element),
      id,
      application,
      frame_revision: Arc::new(AtomicU64::new(0)),
      commanded_frames: Arc::new(Mutex::new(None)),
    }
  }

  /// Implements [`NativeWindow::id`].
  pub(crate) fn id(&self) -> WindowId {
    self.id
  }

  /// Implements [`NativeWindow::title`].
  pub(crate) fn title(&self) -> crate::Result<String> {
    self.element.with(|el| {
      el.get_attribute::<CFString>("AXTitle")
        .map(|cf_string| cf_string.to_string())
    })?
  }

  /// Implements [`NativeWindow::process_name`].
  pub(crate) fn process_name(&self) -> crate::Result<String> {
    self
      .application
      .process_name()
      .ok_or(crate::Error::Platform(
        "Failed to get application process name.".to_string(),
      ))
  }

  /// Implements [`NativeWindow::frame`].
  pub(crate) fn frame(&self) -> crate::Result<Rect> {
    // TODO: Consider refactoring this to use a single dispatch.
    // TODO: Would `AXFrame` work instead?
    let size = self.size()?;
    let position = self.position()?;
    #[allow(clippy::cast_possible_truncation)]
    Ok(Rect::from_xy(
      position.0 as i32,
      position.1 as i32,
      size.0 as i32,
      size.1 as i32,
    ))
  }

  /// Implements [`NativeWindow::position`].
  pub(crate) fn position(&self) -> crate::Result<(f64, f64)> {
    self.element.with(move |el| {
      el.get_attribute::<AXValue>("AXPosition")
        .and_then(|ax_value| ax_value.value_strict::<CGPoint>())
        .map(|point| (point.x, point.y))
    })?
  }

  /// Implements [`NativeWindow::size`].
  pub(crate) fn size(&self) -> crate::Result<(f64, f64)> {
    self.element.with(move |el| {
      el.get_attribute::<AXValue>("AXSize")
        .and_then(|ax_value| ax_value.value_strict::<CGSize>())
        .map(|size| (size.width, size.height))
    })?
  }

  /// Implements [`NativeWindow::is_valid`].
  pub(crate) fn is_valid(&self) -> bool {
    // Query `AXRole`, which is present on all valid `AXUIElement`s.
    self
      .element
      .with(|el| match el.get_attribute::<CFString>("AXRole") {
        Err(crate::Error::Accessibility(_, code))
          if code == AXError::InvalidUIElement.0 =>
        {
          let has_login_window = NSWorkspace::sharedWorkspace()
            .frontmostApplication()
            .and_then(|app| app.bundleIdentifier())
            .is_some_and(|id| id.to_string() == "com.apple.loginwindow");

          // AX calls transiently fail with `InvalidUIElement` during
          // sleep/wake. The window should still be considered valid.
          //
          // Events during sleep:
          //   1. Display goes asleep.
          //   2. AX calls fail with `InvalidUIElement`.
          //   3. Login window activates.
          //
          // Events during wake:
          //   1. Display wakes up.
          //   2. Login window deactivates and AX calls succeed again.
          //
          // Perf: `CGDisplayIsAsleep` ~1-5µs, login window check ~1-2ms.
          CGDisplayIsAsleep(0) || has_login_window
        }
        _ => true,
      })
      .unwrap_or(false)
  }

  /// Implements [`NativeWindow::is_visible`].
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn is_visible(&self) -> crate::Result<bool> {
    Ok(!self.application.is_hidden())
  }

  /// Implements [`NativeWindow::is_minimized`].
  pub(crate) fn is_minimized(&self) -> crate::Result<bool> {
    self.element.with(|el| {
      el.get_attribute::<CFBoolean>("AXMinimized")
        .map(|cf_bool| cf_bool.value())
    })?
  }

  /// Implements [`NativeWindow::is_maximized`].
  pub(crate) fn is_maximized(&self) -> crate::Result<bool> {
    self.element.with(|el| {
      el.get_attribute::<CFBoolean>("AXFullScreen")
        .map(|cf_bool| cf_bool.value())
    })?
  }

  /// Implements [`NativeWindow::is_resizable`].
  pub(crate) fn is_resizable(&self) -> crate::Result<bool> {
    self.element.with(|el| {
      let mut settable = 0;
      // SAFETY: The output points to a live stack byte for this call.
      let result = unsafe {
        el.is_attribute_settable(
          &CFString::from_str("AXSize"),
          std::ptr::NonNull::from(&mut settable),
        )
      };
      resolve_resizability(result, settable != 0)
    })?
  }

  /// Implements [`NativeWindow::is_desktop_window`].
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn is_desktop_window(&self) -> crate::Result<bool> {
    Ok(
      self.application.bundle_id() == Some("com.apple.finder".to_string()),
    )
  }

  /// Implements [`NativeWindow::set_frame`].
  pub(crate) fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    let revision = self.next_frame_revision();
    self.set_frame_at_revision(rect, revision, false, None)
  }

  /// Applies the same monitor containment to initial and delayed writes.
  pub(crate) fn set_frame_within(
    &self,
    rect: &Rect,
    bounds: &Rect,
    retry: bool,
  ) -> crate::Result<()> {
    let rect = Rect::from_xy(
      rect.x(),
      rect.y(),
      rect.width().max(1),
      rect.height().max(1),
    )
    .clamp_size(bounds.width(), bounds.height())
    .translate_within(bounds);
    let revision = self.next_frame_revision();
    let result =
      self.set_frame_at_revision(&rect, revision, retry, Some(bounds));
    if retry {
      let native = self.clone();
      let bounds = bounds.clone();
      tokio::task::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if let Err(error) = native.set_frame_at_revision(
          &rect,
          revision,
          true,
          Some(&bounds),
        ) {
          tracing::warn!("Failed to retry macOS window frame: {error}");
        }
      });
    }
    result
  }

  pub(crate) fn is_programmatic_frame(&self, frame: &Rect) -> bool {
    self
      .commanded_frames
      .lock()
      .unwrap()
      .as_ref()
      .is_some_and(|frames| frames.contains(frame))
  }

  fn remember_frame(&self, requested: &Rect, committed: &Rect) {
    *self.commanded_frames.lock().unwrap() =
      Some([requested.clone(), committed.clone()]);
  }

  /// Keep floating windows at their own placement while containing them.
  pub(crate) fn constrain_to_bounds(
    &self,
    bounds: &Rect,
  ) -> crate::Result<Rect> {
    self.reconcile_frame(None, bounds)
  }

  /// Re-anchor a tile without retrying a native size the app refused.
  pub(crate) fn reconcile_tiling_frame(
    &self,
    requested: &Rect,
    bounds: &Rect,
  ) -> crate::Result<Rect> {
    self.reconcile_frame(Some(requested), bounds)
  }

  fn reconcile_frame(
    &self,
    requested: Option<&Rect>,
    bounds: &Rect,
  ) -> crate::Result<Rect> {
    self.with_enhanced_ui_disabled(|el| {
      let before = self.committed_frame()?;
      let frame = contain_native_frame(
        bounds,
        requested,
        || self.committed_frame(),
        |frame| {
          let point = CGPoint::new(frame.x().into(), frame.y().into());
          el.set_attribute("AXPosition", &AXValue::new_strict(&point)?)
        },
      )?;
      if frame != before {
        self.remember_frame(&frame, &frame);
      }
      Ok(frame)
    })
  }

  /// Invalidates delayed frame writes when a drag or lifecycle change
  /// wins.
  pub(crate) fn cancel_pending_frame_retry(&self) {
    self.next_frame_revision();
  }

  /// Returns the generation owned by a new geometry request.
  fn next_frame_revision(&self) -> u64 {
    self.frame_revision.fetch_add(1, Ordering::SeqCst) + 1
  }

  /// Checks freshness on the AX thread before applying delayed geometry.
  fn set_frame_at_revision(
    &self,
    rect: &Rect,
    revision: u64,
    force_size: bool,
    bounds: Option<&Rect>,
  ) -> crate::Result<()> {
    let rect = rect.clone();
    self.with_enhanced_ui_disabled(move |el| -> crate::Result<()> {
      apply_frame_at_revision(&self.frame_revision, revision, || {
        let current = self.committed_frame().ok();
        let write_frame = || {
          apply_frame(current.as_ref(), &rect, force_size, |attribute| {
            match attribute {
              FrameAttribute::Size => {
                let size =
                  CGSize::new(rect.width().into(), rect.height().into());
                el.set_attribute("AXSize", &AXValue::new_strict(&size)?)
              }
              FrameAttribute::Position => {
                let point = CGPoint::new(rect.x().into(), rect.y().into());
                el.set_attribute(
                  "AXPosition",
                  &AXValue::new_strict(&point)?,
                )
              }
            }
          })
        };
        let committed = if let Some(bounds) = bounds {
          Some(place_visible_frame(
            &rect,
            bounds,
            write_frame,
            || self.committed_frame(),
            |frame| {
              let point = CGPoint::new(frame.x().into(), frame.y().into());
              el.set_attribute("AXPosition", &AXValue::new_strict(&point)?)
            },
          )?)
        } else {
          write_frame()?;
          self.committed_frame().ok()
        };
        if let Some(frame) = committed {
          self.remember_frame(&rect, &frame);
        }
        Ok(())
      })
    })
  }

  /// Reads committed geometry; AX attributes can echo a rejected request.
  pub(crate) fn committed_frame(&self) -> crate::Result<Rect> {
    let mut bounds = CGRect::default();
    let result = unsafe {
      ffi::SLSGetWindowBounds(
        ffi::SLSMainConnectionID(),
        self.id.0,
        &raw mut bounds,
      )
    };
    if result != CGError::Success {
      return Err(crate::Error::Platform(
        "Failed to read committed window bounds.".to_string(),
      ));
    }
    #[allow(clippy::cast_possible_truncation)]
    Ok(Rect::from_xy(
      bounds.origin.x.round() as i32,
      bounds.origin.y.round() as i32,
      bounds.size.width.round() as i32,
      bounds.size.height.round() as i32,
    ))
  }

  /// Reads committed size for fixed-dimension detection.
  fn committed_size(&self) -> crate::Result<(i32, i32)> {
    let frame = self.committed_frame()?;
    Ok((frame.width(), frame.height()))
  }

  /// Verifies both resize directions before treating a dimension as fixed.
  pub(crate) fn detect_fixed_size(
    &self,
    requested: &Rect,
    known: FixedWindowSize,
  ) -> crate::Result<FixedWindowSize> {
    let original = self.committed_size()?;
    let probe_width =
      known.width.is_none() && (original.0 - requested.width()).abs() > 1;
    let probe_height = known.height.is_none()
      && (original.1 - requested.height()).abs() > 1;
    if !probe_width && !probe_height {
      return Ok(known);
    }

    // Give the owning app time to commit a refused/limited resize. Most
    // windows match immediately and never enter this path.
    std::thread::sleep(std::time::Duration::from_millis(100));
    let original = self.committed_size()?;
    let probe_width =
      known.width.is_none() && (original.0 - requested.width()).abs() > 1;
    let probe_height = known.height.is_none()
      && (original.1 - requested.height()).abs() > 1;
    if !probe_width && !probe_height {
      return Ok(known);
    }

    // A pending resize may temporarily leave AX and WindowServer out of
    // sync. Don't infer constraints from that intermediate state.
    ensure_size_settled(self.size()?, original)?;

    let original_frame = self.committed_frame()?;
    self.with_enhanced_ui_disabled(move |el| {
      let original_size =
        CGSize::new(original.0.into(), original.1.into());
      let original_value = AXValue::new_strict(&original_size)?;
      let sample = |delta: i32| -> crate::Result<(i32, i32)> {
        let size = CGSize::new(
          f64::from(if probe_width {
            (original.0 + delta).max(1)
          } else {
            original.0
          }),
          f64::from(if probe_height {
            (original.1 + delta).max(1)
          } else {
            original.1
          }),
        );
        el.set_attribute("AXSize", &AXValue::new_strict(&size)?)?;
        // This only runs for a refused tile dimension, not on every
        // resize. The owning app must commit its size before we sample it.
        std::thread::sleep(std::time::Duration::from_millis(100));
        let committed = self.committed_size()?;
        let ax_size = el
          .get_attribute::<AXValue>("AXSize")?
          .value_strict::<CGSize>()?;
        ensure_size_settled((ax_size.width, ax_size.height), committed)?;
        Ok(committed)
      };
      let probes = (|| {
        let larger = sample(100)?;
        let smaller = sample(-100)?;
        Ok::<_, crate::Error>(FixedWindowSize::from_probes(
          original,
          larger,
          smaller,
          probe_width,
          probe_height,
        ))
      })();
      // Restore even if either probe failed. Restoration failures must not
      // be hidden, and an inconclusive probe must not freeze a dimension.
      el.set_attribute("AXSize", &original_value)?;
      let point =
        CGPoint::new(original_frame.x().into(), original_frame.y().into());
      el.set_attribute("AXPosition", &AXValue::new_strict(&point)?)?;
      self.remember_frame(requested, &self.committed_frame()?);
      let detected = probes?;
      Ok(FixedWindowSize {
        width: known.width.or(detected.width),
        height: known.height.or(detected.height),
        minimum_width: known.minimum_width.or(detected.minimum_width),
        minimum_height: known.minimum_height.or(detected.minimum_height),
      })
    })
  }

  /// Implements [`NativeWindow::resize`].
  pub(crate) fn resize(
    &self,
    width: i32,
    height: i32,
  ) -> crate::Result<()> {
    self.cancel_pending_frame_retry();
    self.with_enhanced_ui_disabled(move |el| -> crate::Result<()> {
      let ax_size = CGSize::new(width.into(), height.into());
      let ax_value = AXValue::new_strict(&ax_size)?;
      el.set_attribute("AXSize", &ax_value)
    })
  }

  /// Implements [`NativeWindow::reposition`].
  pub(crate) fn reposition(&self, x: i32, y: i32) -> crate::Result<()> {
    self.cancel_pending_frame_retry();
    self.with_enhanced_ui_disabled(move |el| -> crate::Result<()> {
      let ax_point = CGPoint::new(x.into(), y.into());
      let ax_value = AXValue::new_strict(&ax_point)?;
      el.set_attribute("AXPosition", &ax_value)
    })
  }

  /// Implements [`NativeWindow::minimize`].
  pub(crate) fn minimize(&self) -> crate::Result<()> {
    self.cancel_pending_frame_retry();
    self.element.with(move |el| -> crate::Result<()> {
      let ax_bool = CFBoolean::new(true);
      el.set_attribute::<CFBoolean>("AXMinimized", &ax_bool.into())
    })?
  }

  /// Implements [`NativeWindow::maximize`].
  pub(crate) fn maximize(&self) -> crate::Result<()> {
    self.element.with(move |el| -> crate::Result<()> {
      let ax_bool = CFBoolean::new(true);
      el.set_attribute::<CFBoolean>("AXFullScreen", &ax_bool.into())
    })?
  }

  /// Implements [`NativeWindow::focus`].
  pub(crate) fn focus(&self) -> crate::Result<()> {
    let psn = self.application.psn()?;
    self.set_front_process(&psn)?;
    self.set_key_window(&psn)?;
    self.raise()
  }

  /// Implements [`NativeWindow::close`].
  pub(crate) fn close(&self) -> crate::Result<()> {
    self.cancel_pending_frame_retry();
    self.element.with(|el| -> crate::Result<()> {
      let close_button =
        el.get_attribute::<AXUIElement>("AXCloseButton")?;

      // Simulate pressing the window's close button.
      let result = unsafe {
        close_button.perform_action(&CFString::from_str("AXPress"))
      };

      if result != AXError::Success {
        return Err(crate::Error::Accessibility(
          "AXPress".to_string(),
          result.0,
        ));
      }

      Ok(())
    })?
  }

  /// Executes a callback with the `AXEnhancedUserInterface` attribute
  /// temporarily disabled on the application `AXUIElement`.
  ///
  /// This is to prevent inconsistent window resizing and repositioning
  /// for certain applications (e.g. Firefox).
  ///
  /// References:
  /// - <https://github.com/koekeishiya/yabai/commit/3fe4c77b001e1a4f613c26f01ea68c0f09327f3a>
  /// - <https://github.com/rxhanson/Rectangle/pull/285>
  fn with_enhanced_ui_disabled<F, R>(
    &self,
    callback: F,
  ) -> crate::Result<R>
  where
    F: FnOnce(&CFRetained<AXUIElement>) -> crate::Result<R> + Send,
    R: Send,
  {
    self.application.ax_element.with(|app_el| {
      // Get whether enhanced UI is currently enabled.
      let was_enabled = app_el
        .get_attribute::<CFBoolean>("AXEnhancedUserInterface")
        .is_ok_and(|cf_bool| cf_bool.value());

      // Disable enhanced UI if it was enabled.
      if was_enabled {
        let ax_bool = CFBoolean::new(false);
        let _ = app_el.set_attribute::<CFBoolean>(
          "AXEnhancedUserInterface",
          &ax_bool.into(),
        );
      }

      // Execute the callback with the window element.
      let result = self.element.with(callback);

      // Restore enhanced UI if it was originally enabled.
      if was_enabled {
        let ax_bool = CFBoolean::new(true);
        let _ = app_el.set_attribute::<CFBoolean>(
          "AXEnhancedUserInterface",
          &ax_bool.into(),
        );
      }

      result
    })??
  }

  fn raise(&self) -> crate::Result<()> {
    self.element.with(move |el| -> crate::Result<()> {
      // This has a couple of caveats:
      // - Some windows do not get raised without first calling
      //   `_SLPSSetFrontProcessWithOptions`.
      // - This changes focus if raising a window of the frontmost (active)
      //   application. For example, if 2 Chrome windows are open and one
      //   is focused, raising the other will change focus to the other
      //   window.
      //
      // Because of these caveats, this method is not exposed as a public
      // API. It's also the reason why the GlazeWM feature of bringing all
      // tiling/floating windows to the front on focus change is not
      // implemented for macOS.
      let result =
        unsafe { el.perform_action(&CFString::from_str("AXRaise")) };

      if result != AXError::Success {
        return Err(crate::Error::Accessibility(
          "AXRaise".to_string(),
          result.0,
        ));
      }

      Ok(())
    })?
  }

  fn set_front_process(
    &self,
    psn: &ffi::ProcessSerialNumber,
  ) -> crate::Result<()> {
    let result = unsafe {
      #[allow(clippy::cast_possible_wrap)]
      ffi::_SLPSSetFrontProcessWithOptions(
        psn,
        self.id.0 as i32,
        ffi::CPS_USER_GENERATED,
      )
    };

    if result != CGError::Success {
      return Err(crate::Error::Platform(
        "Failed to set front process.".to_string(),
      ));
    }

    Ok(())
  }

  fn set_key_window(
    &self,
    psn: &ffi::ProcessSerialNumber,
  ) -> crate::Result<()> {
    // Ref: https://github.com/Hammerspoon/hammerspoon/issues/370#issuecomment-545545468
    let window_id = self.id.0.to_ne_bytes();
    let mut event1 = [0; 0x100];
    event1[0x04] = 0xf8;
    event1[0x08] = 0x01;
    event1[0x3a] = 0x10;
    event1[0x3c..(0x3c + window_id.len())].copy_from_slice(&window_id);
    event1[0x20..(0x20 + 0x10)].fill(0xff);

    let mut event2 = event1;
    event2[0x08] = 0x02;

    for event in [event1, event2] {
      let result =
        unsafe { ffi::SLPSPostEventRecordTo(psn, event.as_ptr().cast()) };

      if result != CGError::Success {
        return Err(crate::Error::Platform(
          "Failed to set key window.".to_string(),
        ));
      }
    }

    Ok(())
  }
}

impl From<NativeWindow> for crate::NativeWindow {
  fn from(window: NativeWindow) -> Self {
    crate::NativeWindow { inner: window }
  }
}

/// Implements [`Dispatcher::visible_windows`].
pub(crate) fn visible_windows(
  dispatcher: &Dispatcher,
) -> crate::Result<Vec<crate::NativeWindow>> {
  Ok(
    platform_impl::all_applications(dispatcher)?
      .iter()
      .filter_map(|app| app.windows().ok())
      .flat_map(std::iter::IntoIterator::into_iter)
      .collect(),
  )
}

/// Implements [`Dispatcher::window_by_id`].
pub(crate) fn window_by_id(
  id: WindowId,
  dispatcher: &Dispatcher,
) -> crate::Result<Option<crate::NativeWindow>> {
  // TODO: The performance of this is terrible. A better solution would be
  // to have a cache of window ID <-> `NativeWindow` instances.
  for app in platform_impl::all_applications(dispatcher)? {
    if let Ok(windows) = app.windows() {
      if let Some(win) = windows.into_iter().find(|w| w.id() == id) {
        return Ok(Some(win));
      }
    }
  }

  Ok(None)
}

/// Implements [`Dispatcher::window_from_point`].
pub(crate) fn window_from_point(
  point: &Point,
  dispatcher: &Dispatcher,
) -> crate::Result<Option<crate::NativeWindow>> {
  // Get the top-most window ID at the given point.
  let window_id = dispatcher.dispatch_sync(|| {
    let cg_point = CGPoint {
      x: f64::from(point.x),
      y: f64::from(point.y),
    };

    let window_id = unsafe {
      NSWindow::windowNumberAtPoint_belowWindowWithWindowNumber(
        cg_point,
        // 0 for all windows.
        0,
        MainThreadMarker::new_unchecked(),
      )
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    WindowId(window_id as u32)
  })?;

  // No window found at the given point.
  if window_id.0 == 0 {
    return Ok(None);
  }

  window_by_id(window_id, dispatcher)
    .map_err(|_| crate::Error::WindowNotFound)
}

/// Implements [`Dispatcher::focused_window`].
pub(crate) fn focused_window(
  dispatcher: &Dispatcher,
) -> crate::Result<crate::NativeWindow> {
  dispatcher
    .dispatch_sync(|| {
      // Get the frontmost (active) application.
      let frontmost_app = NSWorkspace::sharedWorkspace()
        .frontmostApplication()
        .map(|app| Application::new(app, dispatcher.clone()));

      // Get the focused window of the frontmost application.
      frontmost_app.and_then(|app| app.focused_window().ok().flatten())
    })?
    .ok_or(crate::Error::WindowNotFound)
}

/// Implements [`Dispatcher::reset_focus`].
// TODO: Move this to a better-suited module.
pub(crate) fn reset_focus(dispatcher: &Dispatcher) -> crate::Result<()> {
  let Some(application) = platform_impl::application_for_bundle_id(
    "com.apple.finder",
    dispatcher,
  )?
  else {
    return Err(crate::Error::Platform(
      "Failed to get desktop application.".to_string(),
    ));
  };

  let success = application.ns_app.activateWithOptions(
    NSApplicationActivationOptions::ActivateAllWindows,
  );

  if !success {
    return Err(crate::Error::Platform(
      "Failed to activate desktop application.".to_string(),
    ));
  }

  Ok(())
}

#[cfg(test)]
mod frame_tests {
  use super::*;

  #[test]
  fn non_resizable_utility_windows_are_not_given_flexible_tiles() {
    // Both Mac Mouse Fix and NetBird report success with AXSize read-only.
    assert!(!resolve_resizability(AXError::Success, false).unwrap());
    assert!(resolve_resizability(AXError::Success, true).unwrap());
  }

  #[test]
  fn failed_capability_query_does_not_classify_a_window_as_fixed() {
    assert!(resolve_resizability(AXError::CannotComplete, false).is_err());
    assert!(
      resolve_resizability(AXError::AttributeUnsupported, false).unwrap()
    );
  }

  #[test]
  fn unsettled_size_probe_can_retry_after_the_app_commits() {
    // AX reports the tile request while WindowServer still has the old
    // native minimum. Returning success here would cache that request.
    assert!(ensure_size_settled((323.0, 1067.0), (700, 1067)).is_err());
    assert!(ensure_size_settled((700.0, 1067.0), (700, 1067)).is_ok());
  }

  #[test]
  fn rejected_size_write_still_moves_window_to_its_assigned_monitor() {
    use std::cell::RefCell;
    let bounds = Rect::from_xy(1728, -120, 2560, 1390);
    let native = RefCell::new(Rect::from_xy(100, 45, 700, 600));
    let requested = Rect::from_xy(4000, 45, 288, 600);
    let actual = place_visible_frame(
      &requested,
      &bounds,
      || Err(crate::Error::Platform("App refused AXSize".into())),
      || Ok(native.borrow().clone()),
      |frame| {
        let accepted = native.borrow().clone();
        *native.borrow_mut() = Rect::from_xy(
          frame.x(),
          frame.y(),
          accepted.width(),
          accepted.height(),
        );
        Ok(())
      },
    )
    .unwrap();
    assert_eq!(actual, Rect::from_xy(3588, 45, 700, 600));
    assert!(bounds.contains_rect(&actual));
  }

  #[test]
  fn floating_position_inside_the_monitor_is_preserved() {
    let bounds = Rect::from_xy(5, 45, 1718, 1067);
    let floating = Rect::from_xy(508, 45, 938, 1067);
    let actual = contain_native_frame(
      &bounds,
      None,
      || Ok(floating.clone()),
      |_| {
        panic!("An in-bounds floating placement must not be re-anchored")
      },
    )
    .unwrap();
    assert_eq!(actual, floating);
  }

  #[test]
  fn slack_position_drift_does_not_leave_a_strip_on_the_right() {
    use std::cell::{Cell, RefCell};
    let bounds = Rect::from_xy(5, 45, 1718, 1067);
    let requested = Rect::from_xy(785, 45, 938, 1067);
    let native = RefCell::new(Rect::from_xy(508, 45, 938, 1067));
    let moves = Cell::new(0);
    for _ in 0..3 {
      place_visible_frame(
        &requested,
        &bounds,
        || Ok(()),
        || Ok(native.borrow().clone()),
        |frame| {
          moves.set(moves.get() + 1);
          *native.borrow_mut() = frame.clone();
          Ok(())
        },
      )
      .unwrap();
    }
    assert_eq!(*native.borrow(), requested);
    assert_eq!(moves.get(), 1);
  }

  #[test]
  fn restoring_tile_origin_preserves_native_minimum_and_monitor_bounds() {
    use std::cell::RefCell;
    for (bounds, requested, actual, expected) in [
      (
        Rect::from_xy(5, 45, 1718, 1067),
        Rect::from_xy(1400, 45, 323, 1067),
        Rect::from_xy(508, 45, 700, 1067),
        Rect::from_xy(1023, 45, 700, 1067),
      ),
      (
        Rect::from_xy(-2560, -200, 2560, 1440),
        Rect::from_xy(-700, 800, 700, 440),
        Rect::from_xy(-1800, -100, 700, 600),
        Rect::from_xy(-700, 640, 700, 600),
      ),
    ] {
      let native = RefCell::new(actual);
      let frame = place_visible_frame(
        &requested,
        &bounds,
        || Ok(()),
        || Ok(native.borrow().clone()),
        |frame| {
          *native.borrow_mut() = frame.clone();
          Ok(())
        },
      )
      .unwrap();
      assert_eq!(frame, expected);
      assert!(bounds.contains_rect(&frame));
    }
  }

  #[test]
  fn refused_resize_is_contained_using_committed_size() {
    use std::cell::RefCell;
    let bounds = Rect::from_xy(5, 45, 1718, 1067);
    let requested = Rect::from_xy(1400, 45, 323, 1067);
    let native = RefCell::new(Rect::from_xy(5, 45, 700, 1067));
    apply_frame(None, &requested, false, |attribute| {
      if attribute == FrameAttribute::Position {
        *native.borrow_mut() =
          Rect::from_xy(requested.x(), requested.y(), 700, 1067);
      }
      // Model an app refusing AXSize while accepting AXPosition.
      Ok(())
    })
    .unwrap();
    let actual = contain_native_frame(
      &bounds,
      None,
      || Ok(native.borrow().clone()),
      |frame| {
        *native.borrow_mut() = frame.clone();
        Ok(())
      },
    )
    .unwrap();
    assert_eq!(actual, Rect::from_xy(1023, 45, 700, 1067));
    assert!(bounds.contains_rect(&actual));
  }

  #[test]
  fn delayed_app_growth_is_contained_without_repeated_resizes() {
    use std::cell::{Cell, RefCell};
    let bounds = Rect::from_xy(1728, -120, 2560, 1390);
    let native = RefCell::new(Rect::from_xy(4000, 900, 700, 600));
    let moves = Cell::new(0);
    for _ in 0..3 {
      contain_native_frame(
        &bounds,
        None,
        || Ok(native.borrow().clone()),
        |frame| {
          moves.set(moves.get() + 1);
          *native.borrow_mut() = frame.clone();
          Ok(())
        },
      )
      .unwrap();
    }
    assert_eq!(moves.get(), 1);
    assert_eq!(*native.borrow(), Rect::from_xy(3588, 670, 700, 600));
  }

  #[test]
  fn failed_containment_write_is_reported() {
    let result = contain_native_frame(
      &Rect::from_xy(0, 0, 1000, 800),
      None,
      || Ok(Rect::from_xy(900, 0, 700, 600)),
      |_| Err(crate::Error::Platform("Injected position failure".into())),
    );
    assert!(result.is_err());
  }

  #[test]
  fn commands_write_only_the_attributes_they_need() {
    let original = Rect::from_xy(10, 20, 800, 600);
    for (current, requested, forced, expected) in [
      (Some(&original), original.clone(), false, vec![]),
      (
        Some(&original),
        Rect::from_xy(30, 40, 800, 600),
        false,
        vec![FrameAttribute::Position],
      ),
      (
        Some(&original),
        Rect::from_xy(30, 40, 900, 700),
        false,
        vec![
          FrameAttribute::Size,
          FrameAttribute::Position,
          FrameAttribute::Size,
        ],
      ),
      (
        Some(&original),
        original.clone(),
        true,
        vec![
          FrameAttribute::Size,
          FrameAttribute::Position,
          FrameAttribute::Size,
        ],
      ),
      (
        None,
        original.clone(),
        false,
        vec![
          FrameAttribute::Size,
          FrameAttribute::Position,
          FrameAttribute::Size,
        ],
      ),
    ] {
      let mut writes = Vec::new();
      apply_frame(current, &requested, forced, |attribute| {
        writes.push(attribute);
        Ok(())
      })
      .unwrap();
      assert_eq!(writes, expected);
    }
  }

  #[test]
  fn failed_frame_writes_stop_and_propagate() {
    let mut writes = Vec::new();
    let result = apply_frame(
      None,
      &Rect::from_xy(0, 0, 800, 600),
      false,
      |attribute| {
        writes.push(attribute);
        Err(crate::Error::Platform("Injected failure".into()))
      },
    );
    assert!(result.is_err());
    assert_eq!(writes, [FrameAttribute::Size]);
  }

  #[test]
  fn superseded_or_cancelled_retries_cannot_write() {
    let revision = AtomicU64::new(1);
    let mut writes = 0;
    apply_frame_at_revision(&revision, 1, || {
      writes += 1;
      Ok(())
    })
    .unwrap();
    revision.fetch_add(1, Ordering::SeqCst);
    apply_frame_at_revision(&revision, 1, || {
      panic!("stale retry executed")
    })
    .unwrap();
    apply_frame_at_revision(&revision, 2, || {
      writes += 1;
      Ok(())
    })
    .unwrap();
    revision.fetch_add(1, Ordering::SeqCst);
    apply_frame_at_revision(&revision, 2, || {
      panic!("cancelled retry executed")
    })
    .unwrap();
    assert_eq!(writes, 2);
  }
}
