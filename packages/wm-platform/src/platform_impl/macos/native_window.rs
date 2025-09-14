use std::sync::Arc;

use dispatch2::MainThreadBound;
use objc2_application_services::{AXError, AXValue};
use objc2_core_foundation::{
  CFArray, CFBoolean, CFDictionary, CFNumber, CFRetained, CFString,
  CFType, CGPoint, CGSize,
};
use objc2_core_graphics::{
  kCGNullWindowID, kCGWindowName, kCGWindowNumber, kCGWindowOwnerName,
  CGWindowListCopyWindowInfo, CGWindowListOption,
};

use crate::{
  platform_impl::{self, AXUIElement, AXUIElementExt, AXValueExt},
  Dispatcher, Rect, WindowId,
};

/// macOS-specific extensions for `NativeWindow`.
pub trait NativeWindowExtMacOs {
  /// Gets the `AXUIElement` instance for this window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn ax_ui_element(&self) -> &MainThreadBound<CFRetained<AXUIElement>>;

  /// Gets the bundle ID of the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn bundle_id(&self) -> crate::Result<String>;

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
  fn ax_ui_element(&self) -> &MainThreadBound<CFRetained<AXUIElement>> {
    &self.inner.element
  }

  fn bundle_id(&self) -> crate::Result<String> {
    // TODO: This is not correct.
    self.inner.element.get_on_main(|el| {
      el.get_attribute::<CFString>("AXBundleID")
        .map(|cf_string| cf_string.to_string())
    })
  }

  fn role(&self) -> crate::Result<String> {
    self.inner.element.get_on_main(|el| {
      el.get_attribute::<CFString>("AXRole")
        .map(|cf_string| cf_string.to_string())
    })
  }

  fn subrole(&self) -> crate::Result<String> {
    self.inner.element.get_on_main(|el| {
      el.get_attribute::<CFString>("AXSubrole")
        .map(|cf_string| cf_string.to_string())
    })
  }

  fn is_modal(&self) -> crate::Result<bool> {
    self.inner.element.get_on_main(|el| {
      el.get_attribute::<CFBoolean>("AXModal")
        .map(|cf_bool| cf_bool.value())
    })
  }

  fn is_main(&self) -> crate::Result<bool> {
    self.inner.element.get_on_main(|el| {
      el.get_attribute::<CFBoolean>("AXMain")
        .map(|cf_bool| cf_bool.value())
    })
  }
}

#[derive(Clone, Debug)]
pub struct NativeWindow {
  id: WindowId,
  element: Arc<MainThreadBound<CFRetained<AXUIElement>>>,
}

impl NativeWindow {
  /// Creates a new `NativeWindow` instance with the given window handle.
  #[must_use]
  pub fn new(
    id: WindowId,
    element: MainThreadBound<CFRetained<AXUIElement>>,
  ) -> Self {
    Self {
      element: Arc::new(element),
      id,
    }
  }

  pub fn id(&self) -> WindowId {
    self.id
  }

  pub fn title(&self) -> crate::Result<String> {
    self.element.get_on_main(|el| {
      el.get_attribute::<CFString>("AXTitle")
        .map(|cf_string| cf_string.to_string())
    })
  }

  pub fn is_visible(&self) -> crate::Result<bool> {
    // TODO: Implement this properly.
    let minimized = self.element.get_on_main(|el| {
      el.get_attribute::<CFBoolean>("AXMinimized")
        .map(|cf_bool| cf_bool.value())
    })?;

    Ok(!minimized)
  }

  pub fn size(&self) -> crate::Result<(f64, f64)> {
    self.element.get_on_main(move |el| {
      el.get_attribute::<AXValue>("AXSize")
        .and_then(|ax_value| ax_value.value_strict::<CGSize>())
        .map(|size| (size.width, size.height))
    })
  }

  pub fn position(&self) -> crate::Result<(f64, f64)> {
    self.element.get_on_main(move |el| {
      el.get_attribute::<AXValue>("AXPosition")
        .and_then(|ax_value| ax_value.value_strict::<CGPoint>())
        .map(|point| (point.x, point.y))
    })
  }

  pub fn frame(&self) -> crate::Result<Rect> {
    // TODO: Consider refactoring this to use a single dispatch.
    let size = self.size()?;
    let position = self.position()?;
    Ok(Rect::from_xy(
      position.0 as i32,
      position.1 as i32,
      size.0 as i32,
      size.1 as i32,
    ))
  }

  pub fn resize(&self, width: f64, height: f64) -> crate::Result<()> {
    self.element.get_on_main(move |el| -> crate::Result<()> {
      let ax_size = CGSize::new(width, height);
      let ax_value = AXValue::new_strict(&ax_size)?;
      el.set_attribute("AXSize", &ax_value)
    })
  }

  pub fn reposition(&self, x: f64, y: f64) -> crate::Result<()> {
    self.element.get_on_main(move |el| -> crate::Result<()> {
      let ax_point = CGPoint::new(x, y);
      let ax_value = AXValue::new_strict(&ax_point)?;
      el.set_attribute("AXPosition", &ax_value)
    })
  }

  pub fn set_frame(&self, rect: &Rect) -> crate::Result<()> {
    // TODO: Consider refactoring this to use a single dispatch.
    // TODO: Consider adding a separate `set_frame_async` method which
    // spawns a thread. Calling blocking AXUIElement methods from different
    // threads supposedly works fine.
    self.resize(rect.width().into(), rect.height().into())?;
    self.reposition(rect.x().into(), rect.y().into())?;
    self.resize(rect.width().into(), rect.height().into())?;
    Ok(())
  }

  /// Whether the window is minimized.
  pub fn is_minimized(&self) -> crate::Result<bool> {
    self.element.get_on_main(|el| {
      el.get_attribute::<CFBoolean>("AXMinimized")
        .map(|cf_bool| cf_bool.value())
    })
  }

  pub fn minimize(&self) -> crate::Result<()> {
    self.element.get_on_main(move |el| -> crate::Result<()> {
      let ax_bool = CFBoolean::new(true);
      el.set_attribute::<CFBoolean>("AXMinimized", &ax_bool.into())
    })
  }

  pub fn is_maximized(&self) -> crate::Result<bool> {
    self.element.get_on_main(|el| {
      el.get_attribute::<CFBoolean>("AXFullScreen")
        .map(|cf_bool| cf_bool.value())
    })
  }

  pub fn maximize(&self) -> crate::Result<()> {
    self.element.get_on_main(move |el| -> crate::Result<()> {
      let ax_bool = CFBoolean::new(true);
      el.set_attribute::<CFBoolean>("AXFullScreen", &ax_bool.into())
    })
  }

  pub fn close(&self) -> crate::Result<()> {
    self.element.get_on_main(|el| -> crate::Result<()> {
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
    })
  }
}

impl From<NativeWindow> for crate::NativeWindow {
  fn from(window: NativeWindow) -> Self {
    crate::NativeWindow { inner: window }
  }
}

/// Gets all windows on macOS.
///
/// Returns all windows that are on-screen and meet filtering criteria,
/// excluding system windows like Dock, menu bar, and desktop elements.
pub fn all_windows(
  dispatcher: &Dispatcher,
) -> crate::Result<Vec<crate::NativeWindow>> {
  let options = CGWindowListOption::OptionOnScreenOnly
    | CGWindowListOption::ExcludeDesktopElements;

  // let options = CGWindowListOption::ExcludeDesktopElements;

  let window_list: CFRetained<CFArray<CFDictionary<CFString, CFType>>> = unsafe {
    CGWindowListCopyWindowInfo(options, kCGNullWindowID)
      .map(|list| CFRetained::cast_unchecked(list))
      .ok_or(crate::Error::WindowEnumerationFailed)
  }?;

  let mut windows = Vec::new();

  for index in window_list {
    let window_id: CFRetained<CFNumber> = index
      .get(unsafe { kCGWindowNumber })
      .and_then(|window_id| CFRetained::downcast(window_id).ok())
      .ok_or(crate::Error::WindowEnumerationFailed)?;

    println!("window_id: {:?}", window_id);

    if let Some(owner_name) = index
      .get(unsafe { kCGWindowOwnerName })
      .and_then(|owner_name| {
        CFRetained::downcast::<CFString>(owner_name).ok()
      })
    {
      println!("owner_name: {:?}", owner_name);
    }

    if let Some(window_name) =
      index.get(unsafe { kCGWindowName }).and_then(|window_name| {
        CFRetained::downcast::<CFString>(window_name).ok()
      })
    {
      println!("window_name: {:?}", window_name);
    }
  }

  Ok(windows)
}

/// Gets all visible windows from all running applications.
///
/// Returns a vector of `NativeWindow` instances for windows that are
/// currently visible on the current space.
pub fn visible_windows(
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
