use dispatch2::MainThreadBound;
use objc2::{rc::Retained, MainThreadMarker};
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_application_services::AXUIElement;
use objc2_core_foundation::{CFArray, CFRetained};

use crate::{
  platform_impl::{AXUIElementExt, NativeWindow},
  Dispatcher,
};

pub struct Application {
  pub(crate) pid: i32,
  pub(crate) ns_app: Retained<NSRunningApplication>,
  pub(crate) ax_element: MainThreadBound<CFRetained<AXUIElement>>,
}

impl Application {
  pub fn new(
    ns_app: Retained<NSRunningApplication>,
  ) -> crate::Result<Self> {
    let pid = unsafe { ns_app.processIdentifier() };
    let ax_element = MainThreadBound::new(
      unsafe { AXUIElement::new_application(pid) },
      MainThreadMarker::new().unwrap(),
    );

    Ok(Self {
      pid,
      ns_app,
      ax_element,
    })
  }

  pub fn windows(&self) -> crate::Result<Vec<crate::NativeWindow>> {
    self.ax_element.get_on_main(|el| {
      let mtm = MainThreadMarker::new().unwrap();
      let windows = el.get_attribute::<CFArray<AXUIElement>>("AXWindows");

      windows.map(|windows| {
        windows
          .iter()
          .map(|window| {
            let window_element = MainThreadBound::new(window, mtm);
            // TODO: Extract proper CGWindowID from AX element.
            NativeWindow::new(self.pid as isize, window_element).into()
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

    running_apps
      .iter()
      .filter_map(|app| Application::new(app).ok())
      .collect()
  })
}
