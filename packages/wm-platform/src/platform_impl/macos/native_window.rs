use std::sync::Arc;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplicationActivationOptions, NSWindow};
use objc2_application_services::{AXError, AXValue};
use objc2_core_foundation::{
  CFBoolean, CFRetained, CFString, CGPoint, CGSize,
};
use objc2_core_graphics::CGError;

use crate::{
  platform_impl::{
    self, ffi, AXUIElement, AXUIElementExt, AXValueExt, Application,
  },
  Dispatcher, Point, Rect, ThreadBound, WindowId,
};

/// macOS-specific extensions for `NativeWindow`.
pub trait NativeWindowExtMacOs {
  /// Gets the `AXUIElement` instance for this window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn ax_ui_element(&self) -> &ThreadBound<CFRetained<AXUIElement>>;

  /// Gets the bundle ID of the application that owns the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn bundle_id(&self) -> Option<String>;

  /// Gets the role of the window (e.g. `AXWindow`).
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn role(&self) -> crate::Result<String>;

  /// Gets the sub-role of the window (e.g. `AXStandardWindow`).
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn subrole(&self) -> crate::Result<String>;

  /// Whether the window is modal.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn is_modal(&self) -> crate::Result<bool>;

  /// Whether the window is the main window for its application.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn is_main(&self) -> crate::Result<bool>;
}

impl NativeWindowExtMacOs for crate::NativeWindow {
  fn ax_ui_element(&self) -> &ThreadBound<CFRetained<AXUIElement>> {
    &self.inner.element
  }

  fn bundle_id(&self) -> Option<String> {
    self.inner.application.bundle_id()
  }

  fn role(&self) -> crate::Result<String> {
    self.inner.element.with(|el| {
      el.get_attribute::<CFString>("AXRole")
        .map(|cf_string| cf_string.to_string())
    })?
  }

  fn subrole(&self) -> crate::Result<String> {
    self.inner.element.with(|el| {
      el.get_attribute::<CFString>("AXSubrole")
        .map(|cf_string| cf_string.to_string())
    })?
  }

  fn is_modal(&self) -> crate::Result<bool> {
    self.inner.element.with(|el| {
      el.get_attribute::<CFBoolean>("AXModal")
        .map(|cf_bool| cf_bool.value())
    })?
  }

  fn is_main(&self) -> crate::Result<bool> {
    self.inner.element.with(|el| {
      el.get_attribute::<CFBoolean>("AXMain")
        .map(|cf_bool| cf_bool.value())
    })?
  }
}

#[derive(Clone, Debug)]
pub struct NativeWindow {
  id: WindowId,
  element: Arc<ThreadBound<CFRetained<AXUIElement>>>,
  application: Application,
}

impl NativeWindow {
  /// Creates a new `NativeWindow` instance with the given window handle.
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
    }
  }

  pub(crate) fn id(&self) -> WindowId {
    self.id
  }

  pub(crate) fn title(&self) -> crate::Result<String> {
    self.element.with(|el| {
      el.get_attribute::<CFString>("AXTitle")
        .map(|cf_string| cf_string.to_string())
    })?
  }

  pub(crate) fn process_name(&self) -> crate::Result<String> {
    self
      .application
      .process_name()
      .ok_or(crate::Error::Platform(
        "Failed to get application process name.".to_string(),
      ))
  }

  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn is_visible(&self) -> crate::Result<bool> {
    Ok(!self.application.is_hidden())
  }

  pub(crate) fn size(&self) -> crate::Result<(f64, f64)> {
    self.element.with(move |el| {
      el.get_attribute::<AXValue>("AXSize")
        .and_then(|ax_value| ax_value.value_strict::<CGSize>())
        .map(|size| (size.width, size.height))
    })?
  }

  pub(crate) fn position(&self) -> crate::Result<(f64, f64)> {
    self.element.with(move |el| {
      el.get_attribute::<AXValue>("AXPosition")
        .and_then(|ax_value| ax_value.value_strict::<CGPoint>())
        .map(|point| (point.x, point.y))
    })?
  }

  pub(crate) fn frame(&self) -> crate::Result<Rect> {
    // TODO: Consider refactoring this to use a single dispatch.
    // TODO: Would `AXFrame` work instead?
    let size = self.size()?;
    let position = self.position()?;
    Ok(Rect::from_xy(
      position.0 as i32,
      position.1 as i32,
      size.0 as i32,
      size.1 as i32,
    ))
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
        .map(|cf_bool| cf_bool.value())
        .unwrap_or(false);

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

  pub(crate) fn resize(
    &self,
    width: f64,
    height: f64,
  ) -> crate::Result<()> {
    self.with_enhanced_ui_disabled(move |el| -> crate::Result<()> {
      let ax_size = CGSize::new(width, height);
      let ax_value = AXValue::new_strict(&ax_size)?;
      el.set_attribute("AXSize", &ax_value)
    })
  }

  pub(crate) fn reposition(&self, x: f64, y: f64) -> crate::Result<()> {
    self.with_enhanced_ui_disabled(move |el| -> crate::Result<()> {
      let ax_point = CGPoint::new(x, y);
      let ax_value = AXValue::new_strict(&ax_point)?;
      el.set_attribute("AXPosition", &ax_value)
    })
  }

  pub(crate) fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    // TODO: Consider adding a separate `set_frame_async` method which
    // spawns a thread. Calling blocking AXUIElement methods from different
    // threads supposedly works fine.
    // TODO: Refactor the repeated `set_attribute` calls.
    let rect = rect.clone();
    self.with_enhanced_ui_disabled(move |el| -> crate::Result<()> {
      let ax_size = CGSize::new(rect.width().into(), rect.height().into());
      let ax_value = AXValue::new_strict(&ax_size)?;
      el.set_attribute("AXSize", &ax_value)?;
      let ax_point = CGPoint::new(rect.x().into(), rect.y().into());
      let ax_value = AXValue::new_strict(&ax_point)?;
      el.set_attribute("AXPosition", &ax_value)?;
      let ax_size = CGSize::new(rect.width().into(), rect.height().into());
      let ax_value = AXValue::new_strict(&ax_size)?;
      el.set_attribute("AXSize", &ax_value)
    })
  }

  /// Whether the window is minimized.
  pub(crate) fn is_minimized(&self) -> crate::Result<bool> {
    self.element.with(|el| {
      el.get_attribute::<CFBoolean>("AXMinimized")
        .map(|cf_bool| cf_bool.value())
    })?
  }

  pub(crate) fn minimize(&self) -> crate::Result<()> {
    self.element.with(move |el| -> crate::Result<()> {
      let ax_bool = CFBoolean::new(true);
      el.set_attribute::<CFBoolean>("AXMinimized", &ax_bool.into())
    })?
  }

  pub(crate) fn is_maximized(&self) -> crate::Result<bool> {
    self.element.with(|el| {
      el.get_attribute::<CFBoolean>("AXFullScreen")
        .map(|cf_bool| cf_bool.value())
    })?
  }

  pub(crate) fn maximize(&self) -> crate::Result<()> {
    self.element.with(move |el| -> crate::Result<()> {
      let ax_bool = CFBoolean::new(true);
      el.set_attribute::<CFBoolean>("AXFullScreen", &ax_bool.into())
    })?
  }

  pub(crate) fn close(&self) -> crate::Result<()> {
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

  pub(crate) fn is_desktop_window(&self) -> crate::Result<bool> {
    Ok(
      self.application.bundle_id() == Some("com.apple.finder".to_string()),
    )
  }

  pub(crate) fn focus_with_raise(&self) -> crate::Result<()> {
    let psn = self.application.psn()?;
    self.set_front_process(&psn)?;
    self.set_key_window(&psn)?;
    self.raise()
  }

  fn raise(&self) -> crate::Result<()> {
    self.element.with(move |el| -> crate::Result<()> {
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
    let mut event1 = [0u8; 0xf8];
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

/// Gets all visible windows from all running applications.
///
/// Returns a vector of `NativeWindow` instances for windows that are
/// currently visible on the current space.
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

  // Find the corresponding `NativeWindow`.
  window_by_id(window_id, dispatcher)
    .map_err(|_| crate::Error::WindowNotFound)
}

// TODO: Move this to a better-suited module.
pub(crate) fn reset_focus(dispatcher: &Dispatcher) -> crate::Result<()> {
  let Some(application) = platform_impl::application_for_bundle_id(
    dispatcher,
    "com.apple.finder",
  )?
  else {
    return Err(crate::Error::Platform(
      "Failed to get desktop application.".to_string(),
    ));
  };

  let success = unsafe {
    application.ns_app.activateWithOptions(
      NSApplicationActivationOptions::ActivateAllWindows,
    )
  };

  if !success {
    return Err(crate::Error::Platform(
      "Failed to activate desktop application.".to_string(),
    ));
  }

  Ok(())
}
