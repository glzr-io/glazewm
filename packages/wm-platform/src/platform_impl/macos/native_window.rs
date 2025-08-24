use accessibility_sys::kAXWindowsAttribute;
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_application_services::AXValue;
use objc2_core_foundation::{
  CFArray, CFBoolean, CFDictionary, CFNumber, CFRetained, CFString,
  CFType, CGPoint, CGSize,
};
use objc2_core_graphics::{
  kCGNullWindowID, kCGWindowName, kCGWindowNumber, kCGWindowOwnerName,
  CGWindowListCopyWindowInfo, CGWindowListOption,
};
use wm_common::Rect;

use crate::platform_impl::{
  AXUIElement, AXUIElementCreateApplication, AXUIElementExt, AXValueExt,
  EventLoopDispatcher, MainThreadRef,
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

  pub fn reposition(&self, x: f64, y: f64) -> crate::Result<()> {
    self.element.with(move |el| -> crate::Result<()> {
      let ax_point = CGPoint::new(x, y);
      let ax_value = AXValue::new_strict(&ax_point)?;
      el.set_attribute("AXPosition", &ax_value)
    })?
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

/// Gets all running applications and prints the titles of their windows.
///
/// Iterates through all running applications and their AXUIElement
/// windows, printing the title of each window found.
pub fn print_all_app_window_titles(
  dispatcher: &EventLoopDispatcher,
) -> crate::Result<()> {
  let dispatcher_clone = dispatcher.clone();
  dispatcher.dispatch_sync(move || {
    let workspace = unsafe { NSWorkspace::sharedWorkspace() };
    let running_apps = unsafe { workspace.runningApplications() };

    for app in running_apps.iter() {
      let pid = unsafe { app.processIdentifier() };

      // Skip system applications without a bundle identifier
      let bundle_id = unsafe { app.bundleIdentifier() };
      if bundle_id.is_none() {
        continue;
      }

      let bundle_id_str =
        bundle_id.map(|id| id.to_string()).unwrap_or_default();
      println!("Application: {} (PID: {})", bundle_id_str, pid);

      // Create AXUIElement for the application
      let app_element_ref = unsafe { AXUIElementCreateApplication(pid) };

      if app_element_ref.is_null() {
        println!("  Failed to create AXUIElement for application");
        continue;
      }

      let app_element = match AXUIElement::from_ref(app_element_ref) {
        Ok(element) => element,
        Err(e) => {
          println!("  Failed to retain AXUIElement: {}", e);
          continue;
        }
      };

      // Get windows from the application. Note that this fails if
      // accessibility permissions are not granted.
      let windows_result =
        app_element.get_attribute::<CFArray<AXUIElement>>("AXWindows");

      match windows_result {
        Ok(windows_array) => {
          println!("  Found {} windows:", windows_array.len());

          for window in windows_array.iter() {
            // Get the title of each window
            match window.get_attribute::<CFString>("AXTitle") {
              Ok(title) => {
                let title_str = title.to_string();
                if !title_str.is_empty() {
                  println!("    Window: \"{}\"", title_str);
                } else {
                  println!("    Window: (no title)");
                }
              }
              Err(_) => {
                println!("    Window: (failed to get title)");
              }
            }

            let ax_ui_element =
              MainThreadRef::new(dispatcher_clone.clone(), window);

            let native_window = NativeWindow::new(
              1,
              dispatcher_clone.clone(),
              ax_ui_element,
            );

            let bundle_id_str = bundle_id_str.clone();
            std::thread::spawn(move || {
              for i in 1..60 {
                if let Err(e) = native_window
                  .resize(400. + (i as f64 * 10.), 400. + (i as f64 * 10.))
                {
                  println!("  Failed to resize window: {}", e);
                  println!(
                    "Failed to resize window: Application: {} (PID: {})",
                    bundle_id_str, pid
                  );
                }

                std::thread::sleep(std::time::Duration::from_secs(1));
              }
            });
          }
        }
        Err(e) => {
          println!("  Failed to get windows: {}", e);
        }
      }

      println!();
    }

    Ok(())
  })?
}
