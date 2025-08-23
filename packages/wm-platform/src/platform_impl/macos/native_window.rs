use objc2_application_services::AXValue;
use objc2_core_foundation::{
  CFArray, CFBoolean, CFDictionary, CFNumber, CFRetained, CFString,
  CFType, CGSize,
};
use objc2_core_graphics::{
  kCGNullWindowID, kCGWindowNumber, CGWindowListCopyWindowInfo,
  CGWindowListOption,
};

use crate::platform_impl::{
  AXUIElement, AXUIElementExt, AXValueExt, EventLoopDispatcher,
  MainThreadRef,
};

/// macOS-specific extensions for `NativeWindow`.
pub trait NativeWindowExtMacOs {
  /// Gets the `AXUIElement` instance for this window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn ax_ui_element(&self) -> &MainThreadRef<CFRetained<AXUIElement>>;

  /// Gets the bundle ID of the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn bundle_id(&self) -> crate::Result<String>;

  /// Gets the role of the window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  fn role(&self) -> crate::Result<String>;
}

impl NativeWindowExtMacOs for crate::NativeWindow {
  fn ax_ui_element(&self) -> &MainThreadRef<CFRetained<AXUIElement>> {
    &self.inner.element
  }

  fn bundle_id(&self) -> crate::Result<String> {
    self.inner.element.with(|el| {
      el.get_attribute::<CFString>("AXBundleID")
        .map(|cf_string| cf_string.to_string())
    })?
  }

  fn role(&self) -> crate::Result<String> {
    self.inner.element.with(|el| {
      el.get_attribute::<CFString>("AXRole")
        .map(|cf_string| cf_string.to_string())
    })?
  }
}

#[derive(Clone, Debug)]
pub struct NativeWindow {
  element: MainThreadRef<CFRetained<AXUIElement>>,
  dispatcher: EventLoopDispatcher,
  pub handle: isize,
}

impl NativeWindow {
  /// Creates a new `NativeWindow` instance with the given window handle.
  #[must_use]
  pub fn new(
    handle: isize,
    dispatcher: EventLoopDispatcher,
    element: MainThreadRef<CFRetained<AXUIElement>>,
  ) -> Self {
    Self {
      element,
      dispatcher,
      handle,
    }
  }

  pub fn title(&self) -> crate::Result<String> {
    self.element.with(|el| {
      el.get_attribute::<CFString>("AXTitle")
        .map(|r| r.to_string())
    })?
  }

  pub fn is_visible(&self) -> crate::Result<bool> {
    // TODO: Implement this properly.
    let minimized = self.element.with(|el| {
      el.get_attribute::<CFBoolean>("AXMinimized")
        .map(|cf_bool| cf_bool.value())
    })??;

    Ok(!minimized)
  }

  pub fn resize(&self, width: f64, height: f64) -> crate::Result<()> {
    self.element.with(move |el| -> crate::Result<()> {
      let ax_size = CGSize::new(width, height);
      let ax_value = AXValue::new_strict(&ax_size)?;
      el.set_attribute("AXSize", &ax_value)
    })?
  }

  /// Whether the window is minimized.
  pub fn is_minimized(&self) -> crate::Result<bool> {
    self.element.with(|el| {
      el.get_attribute::<CFBoolean>("AXMinimized")
        .map(|cf_bool| cf_bool.value())
    })?
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
  dispatcher: &EventLoopDispatcher,
) -> crate::Result<Vec<crate::NativeWindow>> {
  let options = CGWindowListOption::OptionOnScreenOnly
    | CGWindowListOption::ExcludeDesktopElements;

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
  }

  Ok(windows)
}
