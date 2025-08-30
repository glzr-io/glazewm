use std::ffi::c_void;

use objc2_application_services::AXError;
use objc2_core_foundation::CFString;

use super::ax_ui_element::AXUIElementRef;

pub enum __AXObserver {}
pub type AXObserverRef = *mut __AXObserver;

#[repr(C)]
pub struct __CFRunLoopSource(c_void);

pub type CFRunLoopSourceRef = *mut __CFRunLoopSource;

pub type ProcessId = i32;
pub type CFStringRef = *const CFString;

pub type AXObserverCallback = unsafe extern "C" fn(
  observer: AXObserverRef,
  element: AXUIElementRef,
  notification: CFStringRef,
  refcon: *mut c_void,
);

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
  pub fn AXObserverCreate(
    application: ProcessId,
    callback: AXObserverCallback,
    out_observer: &mut AXObserverRef,
  ) -> AXError;

  pub fn AXObserverAddNotification(
    observer: AXObserverRef,
    element: AXUIElementRef,
    notification: &CFString,
    refcon: *mut c_void,
  ) -> AXError;

  pub fn AXObserverRemoveNotification(
    observer: AXObserverRef,
    element: AXUIElementRef,
    notification: &CFString,
  ) -> AXError;

  pub fn AXUIElementCreateApplication(pid: ProcessId) -> AXUIElementRef;

  pub fn AXObserverGetRunLoopSource(
    observer: AXObserverRef,
  ) -> CFRunLoopSourceRef;
}
