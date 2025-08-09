use std::ffi::c_void;

use accessibility_sys::AXError;
use objc2_core_foundation::{CFString, CFType};

// Opaque CoreFoundation type representing AXUIElement
// AXUIElement is a CFType, not an Objective-C class.
// It follows CFRetain/CFRelease semantics.
pub enum AXUIElement {}
pub type AXUIElementRef = *mut AXUIElement;

// Mark the opaque CF type so it can be used with CFRetained
unsafe impl objc2_core_foundation::Type for AXUIElement {}

pub enum __AXObserver {}
pub type AXObserverRef = *mut __AXObserver;

#[repr(C)]
pub struct __CFRunLoopSource(c_void);

pub type CFRunLoopSourceRef = *mut __CFRunLoopSource;
pub type CGKeyCode = u16;
pub type CGCharCode = u16;

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

  pub fn AXUIElementCopyAttributeValue(
    element: AXUIElementRef,
    attribute: CFStringRef,
    value: &mut *mut CFType,
  ) -> AXError;

  pub fn AXUIElementCreateApplication(pid: ProcessId) -> AXUIElementRef;

  pub fn AXObserverGetRunLoopSource(
    observer: AXObserverRef,
  ) -> CFRunLoopSourceRef;
}
