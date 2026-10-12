use std::sync::{
  atomic::{AtomicU64, Ordering},
  Arc,
};

use objc2::MainThreadMarker;
use objc2_app_kit::{
  NSApplicationActivationOptions, NSWindow, NSWorkspace,
};
use objc2_application_services::{AXError, AXValue};
use objc2_core_foundation::{
  CFArray, CFBoolean, CFDictionary, CFRetained, CFString, CFType, CGPoint,
  CGRect, CGSize,
};
use objc2_core_graphics::{
  kCGWindowName, CGDisplayIsAsleep, CGError, CGWindowListCopyWindowInfo,
  CGWindowListOption,
};

use crate::{
  platform_impl::{
    self, ffi, AXUIElement, AXUIElementExt, AXValueExt, Application,
    BORDER_OVERLAY_MANAGER,
  },
  Color, Dispatcher, FixedWindowSize, Point, Rect, ThreadBound, WindowId,
};

// A window title is optional metadata. Some Chromium app shims expose a
// valid standard window but fail AXTitle reads, even with a visible title.
fn resolve_window_title(
  title: crate::Result<String>,
  fallback: impl FnOnce() -> Option<String>,
) -> crate::Result<String> {
  match title {
    Ok(title) => Ok(title),
    Err(crate::Error::Accessibility(attribute, code))
      if attribute == "AXTitle"
        && [
          AXError::Failure.0,
          AXError::CannotComplete.0,
          AXError::AttributeUnsupported.0,
          AXError::NoValue.0,
        ]
        .contains(&code) =>
    {
      tracing::debug!("AXTitle read failed ({code}); using a Core Graphics title or an empty title.");
      Ok(fallback().unwrap_or_default())
    }
    Err(error) => Err(error),
  }
}

fn core_graphics_window_title(id: WindowId) -> Option<String> {
  let windows = CGWindowListCopyWindowInfo(
    CGWindowListOption::OptionIncludingWindow,
    id.0,
  )?;
  // SAFETY: CGWindowListCopyWindowInfo returns dictionaries with CFString
  // keys and Core Foundation values, retained by the array.
  let windows = unsafe {
    CFRetained::cast_unchecked::<CFArray<CFDictionary<CFString, CFType>>>(
      windows,
    )
  };
  let info = windows.get(0)?;
  let title = info
    .get(unsafe { kCGWindowName }.as_ref())?
    .downcast::<CFString>()
    .ok()?;
  Some(title.to_string())
}

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
    }
  }

  /// Implements [`NativeWindow::id`].
  pub(crate) fn id(&self) -> WindowId {
    self.id
  }

  /// Implements [`NativeWindow::title`].
  pub(crate) fn title(&self) -> crate::Result<String> {
    let title = self.element.with(|el| {
      el.get_attribute::<CFString>("AXTitle")
        .map(|cf_string| cf_string.to_string())
    })?;
    resolve_window_title(title, || {
      // Native tab switches can change the ID of the same AX element.
      let id = self.current_window_id().unwrap_or(self.id);
      core_graphics_window_title(id)
    })
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
    // Read both attributes in one dispatch so a queued command cannot
    // execute between the size and position reads.
    self.element.with(|el| {
      let size = el
        .get_attribute::<AXValue>("AXSize")?
        .value_strict::<CGSize>()?;
      let position = el
        .get_attribute::<AXValue>("AXPosition")?
        .value_strict::<CGPoint>()?;
      #[allow(clippy::cast_possible_truncation)]
      Ok(Rect::from_xy(
        position.x as i32,
        position.y as i32,
        size.width as i32,
        size.height as i32,
      ))
    })?
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

  /// Gets the Core Graphics window layer.
  pub(crate) fn layer(&self) -> crate::Result<i32> {
    let mut layer = 0;
    let result = unsafe {
      ffi::SLSGetWindowLevel(
        ffi::SLSMainConnectionID(),
        self.id.0,
        &raw mut layer,
      )
    };

    if result != CGError::Success {
      return Err(crate::Error::Platform(
        "Failed to get Core Graphics window layer.".to_string(),
      ));
    }

    Ok(layer)
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
  #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
  pub(crate) fn is_resizable(&self) -> crate::Result<bool> {
    // TODO: Not sure if this is even available via the AX API.
    Ok(true)
  }

  /// Implements [`NativeWindow::is_desktop_window`].
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn is_desktop_window(&self) -> crate::Result<bool> {
    Ok(
      self.application.bundle_id() == Some("com.apple.finder".to_string()),
    )
  }

  /// Whether this window is a root window (not a background tab).
  ///
  /// Native macOS tabs share the same physical window frame, but each
  /// tab is a separate `AXUIElement`. A background tab's `AXParent`
  /// points to another `AXWindow` (the active tab), whereas a root
  /// window's `AXParent` points to the `AXApplication`.
  pub(crate) fn is_root_window(&self) -> crate::Result<bool> {
    self.element.with(|el| {
      let parent = el.get_attribute::<AXUIElement>("AXParent")?;
      let parent_role = parent.get_attribute::<CFString>("AXRole")?;

      Ok(parent_role.to_string() != "AXWindow")
    })?
  }

  /// Gets the process ID of the application that owns this window.
  pub(crate) fn process_id(&self) -> platform_impl::ProcessId {
    self.application.pid
  }

  /// Re-queries the current `CGWindowID` from the underlying
  /// `AXUIElement`.
  ///
  /// For native macOS tab groups, the `CGWindowID` changes on tab
  /// switch even though the `AXUIElement` stays the same.
  pub(crate) fn current_window_id(&self) -> crate::Result<WindowId> {
    self.element.with(WindowId::from_window_element)?
  }

  /// Implements [`NativeWindow::set_frame`].
  pub(crate) fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    let revision = self.next_frame_revision();
    self.set_frame_at_revision(rect, revision, false)
  }

  /// Retries a cross-DPI frame only while it is the latest request.
  pub(crate) fn set_frame_with_dpi_retry(
    &self,
    rect: &Rect,
  ) -> crate::Result<()> {
    let revision = self.next_frame_revision();
    let result = self.set_frame_at_revision(rect, revision, true);
    let native = self.clone();
    let rect = rect.clone();
    tokio::task::spawn(async move {
      tokio::time::sleep(std::time::Duration::from_millis(100)).await;
      if let Err(error) =
        native.set_frame_at_revision(&rect, revision, true)
      {
        tracing::warn!("Failed to retry macOS window frame: {error}");
      }
    });
    result
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
  ) -> crate::Result<()> {
    let rect = rect.clone();
    self.with_enhanced_ui_disabled(move |el| -> crate::Result<()> {
      apply_frame_at_revision(&self.frame_revision, revision, || {
        let current = self.committed_frame().ok();
        apply_frame(current.as_ref(), &rect, force_size, |attribute| {
          match attribute {
            FrameAttribute::Size => {
              let size =
                CGSize::new(rect.width().into(), rect.height().into());
              el.set_attribute("AXSize", &AXValue::new_strict(&size)?)
            }
            FrameAttribute::Position => {
              let point = CGPoint::new(rect.x().into(), rect.y().into());
              el.set_attribute("AXPosition", &AXValue::new_strict(&point)?)
            }
          }
        })?;
        if let Ok(frame) = self.committed_frame() {
          if let Err(error) =
            platform_impl::update_border_position(self.id(), &frame)
          {
            tracing::warn!(
              "Failed to sync border after macOS frame change: {error}"
            );
          }
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
    let ax_size = self.size()?;
    if (ax_size.0 - f64::from(original.0)).abs() > 0.5
      || (ax_size.1 - f64::from(original.1)).abs() > 0.5
    {
      return Ok(known);
    }

    self.with_enhanced_ui_disabled(move |el| {
      let original_size = CGSize::new(original.0.into(), original.1.into());
      let original_value = AXValue::new_strict(&original_size)?;
      let sample = |delta: i32| -> crate::Result<(i32, i32)> {
        let size = CGSize::new(
          f64::from(if probe_width { (original.0 + delta).max(1) } else { original.0 }),
          f64::from(if probe_height { (original.1 + delta).max(1) } else { original.1 }),
        );
        el.set_attribute("AXSize", &AXValue::new_strict(&size)?)?;
        // This only runs for a refused tile dimension, not on every resize.
        // The owning app must commit its size before we sample it.
        std::thread::sleep(std::time::Duration::from_millis(100));
        let committed = self.committed_size()?;
        let ax_size = el.get_attribute::<AXValue>("AXSize")?
          .value_strict::<CGSize>()?;
        if (ax_size.width - f64::from(committed.0)).abs() > 0.5
          || (ax_size.height - f64::from(committed.1)).abs() > 0.5
        {
          return Err(crate::Error::Platform(
            "Window resize has not settled; skipping fixed-size detection.".to_string(),
          ));
        }
        Ok(committed)
      };
      let probes = (|| {
        let larger = sample(100)?;
        let smaller = sample(-100)?;
        Ok::<_, crate::Error>(FixedWindowSize::from_probes(
          original, larger, smaller, probe_width, probe_height,
        ))
      })();
      // Restore even if either probe failed. Restoration failures must not
      // be hidden, and an inconclusive probe must not freeze a dimension.
      el.set_attribute("AXSize", &original_value)?;
      let detected = probes?;
      Ok(FixedWindowSize {
        width: known.width.or(detected.width),
        height: known.height.or(detected.height),
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
    self.cancel_pending_frame_retry();
    self.element.with(move |el| -> crate::Result<()> {
      let ax_bool = CFBoolean::new(true);
      el.set_attribute::<CFBoolean>("AXFullScreen", &ax_bool.into())
    })?
  }

  /// Implements [`NativeWindowExtMacOs::unmaximize`].
  pub(crate) fn unmaximize(&self) -> crate::Result<()> {
    self.cancel_pending_frame_retry();
    self.element.with(move |el| -> crate::Result<()> {
      let ax_bool = CFBoolean::new(false);
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

  /// Implements [`NativeWindowExtMacOs::set_border_color`].
  pub(crate) fn set_border_color(
    &self,
    color: Option<&Color>,
  ) -> crate::Result<()> {
    let window_id = self.id();
    // Resolve geometry before locking. No overlay lock may be held
    // across a dispatch to the AX thread.
    let frame = color.map(|_| self.committed_frame()).transpose()?;
    let mut manager = BORDER_OVERLAY_MANAGER.lock().map_err(|_| {
      crate::Error::Platform(
        "Border overlay manager lock poisoned.".to_string(),
      )
    })?;

    let Some(color) = color else {
      manager.remove(window_id);
      return Ok(());
    };

    if let Some(frame) = frame {
      manager.set_border_color(window_id, &frame, Some(color))?;
    }
    Ok(())
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
  // Self-filtering: exclude GlazeWM's own overlay windows.
  let own_pid = std::process::id().cast_signed();

  Ok(
    platform_impl::all_applications(dispatcher)?
      .iter()
      .filter(|app| app.pid != own_pid)
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

#[cfg(test)]
mod title_tests {
  use super::*;

  #[test]
  fn available_accessibility_title_does_not_query_core_graphics() {
    assert_eq!(
      resolve_window_title(Ok("Document".into()), || panic!(
        "unexpected fallback"
      ))
      .unwrap(),
      "Document"
    );
    assert_eq!(
      resolve_window_title(Ok(String::new()), || panic!(
        "unexpected fallback"
      ))
      .unwrap(),
      ""
    );
  }

  #[test]
  fn failed_accessibility_title_uses_visible_window_title() {
    for code in [
      AXError::Failure.0,
      AXError::CannotComplete.0,
      AXError::AttributeUnsupported.0,
      AXError::NoValue.0,
    ] {
      let result = resolve_window_title(
        Err(crate::Error::Accessibility("AXTitle".into(), code)),
        || Some("Brain.fm App".into()),
      );
      assert_eq!(result.unwrap(), "Brain.fm App");
    }
  }

  #[test]
  fn unavailable_optional_titles_do_not_reject_a_window() {
    let result = resolve_window_title(
      Err(crate::Error::Accessibility(
        "AXTitle".into(),
        AXError::Failure.0,
      )),
      || None,
    );
    assert_eq!(result.unwrap(), "");
  }

  #[test]
  fn invalid_window_and_dispatch_errors_are_preserved() {
    for error in [
      crate::Error::Accessibility(
        "AXTitle".into(),
        AXError::InvalidUIElement.0,
      ),
      crate::Error::EventLoopStopped,
    ] {
      assert!(resolve_window_title(Err(error), || panic!(
        "unexpected fallback"
      ))
      .is_err());
    }
  }
}
