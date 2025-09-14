use std::sync::Arc;

use dispatch2::MainThreadBound;
use objc2::{rc::Retained, MainThreadMarker};
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_application_services::AXUIElement;
use objc2_core_foundation::{CFArray, CFRetained};

use crate::{
  platform_impl::{AXUIElementExt, NativeWindow},
  Dispatcher, WindowId,
};

pub type ProcessId = i32;

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
            NativeWindow::new(window_id, window_el).into()
          })
          .collect()
      })
    })
  }
}

pub fn all_applications(
  dispatcher: &Dispatcher,
) -> crate::Result<Vec<Application>> {
  dispatcher.dispatch_sync(move || {
    let running_apps =
      unsafe { NSWorkspace::sharedWorkspace().runningApplications() };

    running_apps.iter().map(Application::new).collect()
  })
}
