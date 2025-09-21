use std::sync::Arc;

use dispatch2::MainThreadBound;
use objc2::{rc::Retained, MainThreadMarker};
use objc2_app_kit::{
  NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace,
};
use objc2_application_services::AXUIElement;
use objc2_core_foundation::{CFArray, CFRetained};
use objc2_foundation::NSString;

use crate::{
  platform_impl::{ffi, AXUIElementExt, NativeWindow},
  Dispatcher, WindowId,
};

pub type ProcessId = i32;

#[derive(Clone, Debug)]
pub struct Application {
  pub(crate) pid: ProcessId,
  pub(crate) ns_app: Retained<NSRunningApplication>,
  pub(crate) ax_element: Arc<MainThreadBound<CFRetained<AXUIElement>>>,
}

impl Application {
  pub(crate) fn new(ns_app: Retained<NSRunningApplication>) -> Self {
    let pid = unsafe { ns_app.processIdentifier() };
    let ax_element = Arc::new(MainThreadBound::new(
      unsafe { AXUIElement::new_application(pid) },
      MainThreadMarker::new().unwrap(),
    ));

    Self {
      pid,
      ns_app,
      ax_element,
    }
  }

  pub fn focused_window(
    &self,
  ) -> crate::Result<Option<crate::NativeWindow>> {
    self.ax_element.get_on_main(|el| {
      let mtm = MainThreadMarker::new().unwrap();
      let focused_window =
        el.get_attribute::<AXUIElement>("AXFocusedWindow");

      focused_window.map(|window_el| {
        let window_id = WindowId::from_window_element(&window_el);
        let window_el = MainThreadBound::new(window_el, mtm);
        Some(NativeWindow::new(window_id, window_el, self.clone()).into())
      })
    })
  }

  pub fn windows(&self) -> crate::Result<Vec<crate::NativeWindow>> {
    self.ax_element.get_on_main(|el| {
      let mtm = MainThreadMarker::new().unwrap();
      let windows = el.get_attribute::<CFArray<AXUIElement>>("AXWindows");

      windows.map(|windows| {
        windows
          .iter()
          .map(|window_el| {
            let window_id = WindowId::from_window_element(&window_el);
            let window_el = MainThreadBound::new(window_el, mtm);
            NativeWindow::new(window_id, window_el, self.clone()).into()
          })
          .collect()
      })
    })
  }

  pub fn psn(&self) -> crate::Result<ffi::ProcessSerialNumber> {
    let mut psn = ffi::ProcessSerialNumber::default();

    if unsafe { ffi::GetProcessForPID(self.pid, &raw mut psn) } != 0 {
      return Err(crate::Error::Platform(
        "Failed to get process serial number.".to_string(),
      ));
    }

    Ok(psn)
  }

  pub fn bundle_id(&self) -> Option<String> {
    unsafe { self.ns_app.bundleIdentifier() }
      .map(|ns_string| ns_string.to_string())
  }

  pub fn process_name(&self) -> Option<String> {
    unsafe { self.ns_app.localizedName() }
      .map(|ns_string| ns_string.to_string())
  }

  /// Whether the application is an XPC service.
  ///
  /// Some of Apple's own XPC services have window capabilities. These
  /// windows are non-standard and unmanageable.
  pub fn is_xpc(&self) -> crate::Result<bool> {
    let psn = self.psn()?;

    let mut info = ffi::ProcessInfo::default();
    info.info_length = std::mem::size_of::<ffi::ProcessInfo>() as u32;

    if unsafe { ffi::GetProcessInformation(&raw const psn, &raw mut info) }
      != 0
    {
      return Err(crate::Error::Platform(
        "Failed to get process information.".to_string(),
      ));
    }

    Ok(info.r#type.to_be_bytes() == *b"XPC!")
  }

  pub fn activation_policy(&self) -> NSApplicationActivationPolicy {
    unsafe { self.ns_app.activationPolicy() }
  }

  /// Whether the application should be observed.
  pub(crate) fn should_observe(&self) -> bool {
    if self.activation_policy()
      == NSApplicationActivationPolicy::Prohibited
    {
      return false;
    }

    !self.is_xpc().unwrap_or(false)
  }

  pub(crate) fn is_hidden(&self) -> bool {
    unsafe { self.ns_app.isHidden() }
  }
}

pub(crate) fn all_applications(
  dispatcher: &Dispatcher,
) -> crate::Result<Vec<Application>> {
  dispatcher.dispatch_sync(move || {
    let running_apps =
      unsafe { NSWorkspace::sharedWorkspace().runningApplications() };

    running_apps.iter().map(Application::new).collect()
  })
}

pub(crate) fn application_for_bundle_id(
  dispatcher: &Dispatcher,
  bundle_id: &str,
) -> crate::Result<Option<Application>> {
  let bundle_id = bundle_id.to_owned();
  dispatcher.dispatch_sync(move || {
    let apps = unsafe {
      NSRunningApplication::runningApplicationsWithBundleIdentifier(
        &NSString::from_str(&bundle_id),
      )
    };

    apps.into_iter().next().map(Application::new)
  })
}
