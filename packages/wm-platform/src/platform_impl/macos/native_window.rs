use std::ptr::NonNull;

use objc2::msg_send;
use objc2_application_services::AXValue;
use objc2_core_foundation::{
  CFBoolean, CFRetained, CFString, CFType, CGSize,
};
use objc2_core_graphics::{
  CGWindowID, CGWindowListCopyWindowInfo, CGWindowListOption,
};
use wm_common::Rect;

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

  let window_list = unsafe { CGWindowListCopyWindowInfo(options, 0) }
    .ok_or(crate::Error::WindowEnumerationFailed)?;

  let mut windows = Vec::new();

  // CFArray length
  let array_len = unsafe {
    let count: usize = msg_send![&*window_list, count];
    count
  };

  for i in 0..array_len {
    // Get object at index
    let window_dict_obj = unsafe {
      let obj: *const CFType = msg_send![&*window_list, objectAtIndex: i];
      if obj.is_null() {
        continue;
      }
      CFRetained::retain(NonNull::new_unchecked(obj as *mut CFType))
    };

    if let Some(window) = parse_window_dict(&window_dict_obj, dispatcher) {
      windows.push(window.into());
    }
  }

  Ok(windows)
}

/// Parses a window dictionary from CGWindowListCopyWindowInfo.
fn parse_window_dict(
  dict_obj: &CFRetained<CFType>,
  dispatcher: &EventLoopDispatcher,
) -> Option<crate::platform_impl::NativeWindow> {
  // Extract window ID - this is the minimum we need
  let window_id =
    get_number_from_dict(dict_obj, "kCGWindowNumber")? as CGWindowID;

  // Extract owner PID to create proper AXUIElement
  let owner_pid =
    get_number_from_dict(dict_obj, "kCGWindowOwnerPID")? as i32;

  // Extract layer to filter system windows
  let layer =
    get_number_from_dict(dict_obj, "kCGWindowLayer").unwrap_or(0.0) as i64;

  // Extract alpha (transparency)
  let alpha =
    get_number_from_dict(dict_obj, "kCGWindowAlpha").unwrap_or(1.0);

  // Filter out desktop elements, dock, menu bar, etc.
  // Layer 0 is normal application windows
  if layer != 0 || alpha < 0.1 {
    return None;
  }

  // Extract window bounds for size filtering
  if let Some(bounds) = get_bounds_from_dict(dict_obj) {
    // Skip very small windows (likely system elements)
    if bounds.width() < 50 || bounds.height() < 50 {
      return None;
    }
  }

  // Create application accessibility element from PID
  let ax_element = unsafe { AXUIElement::new_application(owner_pid) };

  let element_ref = crate::platform_impl::MainThreadRef::new(
    dispatcher.clone(),
    ax_element,
  );

  Some(crate::platform_impl::NativeWindow::new(
    window_id as isize,
    dispatcher.clone(),
    element_ref,
  ))
}

/// Gets a number value from a CFDictionary using msg_send.
fn get_number_from_dict(
  dict_obj: &CFRetained<CFType>,
  key: &str,
) -> Option<f64> {
  let key_string = CFString::from_str(key);

  // Get the value from the dictionary using msg_send
  let value_obj = unsafe {
    let obj: *const CFType =
      msg_send![&**dict_obj, objectForKey: &*key_string];
    if obj.is_null() {
      return None;
    }
    CFRetained::retain(NonNull::new_unchecked(obj as *mut CFType))
  };

  // Try to convert to CFNumber and get f64 value
  unsafe {
    let f64_value: f64 = msg_send![&*value_obj, doubleValue];
    Some(f64_value)
  }
}

/// Gets window bounds from a CFDictionary.
fn get_bounds_from_dict(dict_obj: &CFRetained<CFType>) -> Option<Rect> {
  let bounds_key = CFString::from_str("kCGWindowBounds");

  let bounds_dict = unsafe {
    let obj: *const CFType =
      msg_send![&**dict_obj, objectForKey: &*bounds_key];
    if obj.is_null() {
      return None;
    }
    CFRetained::retain(NonNull::new_unchecked(obj as *mut CFType))
  };

  let x = get_number_from_dict(&bounds_dict, "X")? as i32;
  let y = get_number_from_dict(&bounds_dict, "Y")? as i32;
  let width = get_number_from_dict(&bounds_dict, "Width")? as i32;
  let height = get_number_from_dict(&bounds_dict, "Height")? as i32;

  Some(Rect::from_ltrb(x, y, x + width, y + height))
}
