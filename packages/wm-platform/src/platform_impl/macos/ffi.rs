use std::ffi::c_void;

use accessibility_sys::{
  kAXValueTypeCFRange, kAXValueTypeCGPoint, kAXValueTypeCGRect,
  kAXValueTypeCGSize, AXError, AXValueGetValue,
};
use objc2_core_foundation::{
  CFRange, CFString, CFType, CGPoint, CGRect, CGSize,
};

use super::ax_ui_element::AXUIElementRef;

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

// Opaque AXValue type - this is actually a CFType
#[repr(C)]
pub struct AXValue(c_void);
pub type AXValueRef = *mut AXValue;
pub type AXValueType = u32;

// Mark the opaque CF type so it can be used with CFRetained
unsafe impl objc2_core_foundation::Type for AXValue {}

pub trait AXValueKind {
  const TYPE: AXValueType;
}

impl AXValueKind for CGPoint {
  const TYPE: AXValueType = kAXValueTypeCGPoint;
}
impl AXValueKind for CGSize {
  const TYPE: AXValueType = kAXValueTypeCGSize;
}
impl AXValueKind for CGRect {
  const TYPE: AXValueType = kAXValueTypeCGRect;
}
impl AXValueKind for CFRange {
  const TYPE: AXValueType = kAXValueTypeCFRange;
}

// Helper trait for creating default values
pub trait AXValueDefault {
  fn ax_default() -> Self;
}

impl AXValueDefault for CGPoint {
  fn ax_default() -> Self {
    CGPoint::new(0.0, 0.0)
  }
}

impl AXValueDefault for CGSize {
  fn ax_default() -> Self {
    CGSize::new(0.0, 0.0)
  }
}

impl AXValueDefault for CGRect {
  fn ax_default() -> Self {
    CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(0.0, 0.0))
  }
}

impl AXValueDefault for CFRange {
  fn ax_default() -> Self {
    CFRange {
      location: 0,
      length: 0,
    }
  }
}

impl AXValue {
  pub fn new<T: AXValueKind>(
    val: &T,
  ) -> crate::Result<objc2_core_foundation::CFRetained<Self>> {
    let ptr =
      unsafe { AXValueCreate(T::TYPE, val as *const T as *const c_void) };

    if ptr.is_null() {
      Err(crate::Error::AXValueCreation(
        "AXValueCreate failed".to_string(),
      ))
    } else {
      // Convert raw pointer to CFRetained
      let nn_ptr =
        std::ptr::NonNull::new(ptr as *mut Self).ok_or_else(|| {
          crate::Error::AXValueCreation(
            "AXValueCreate returned null".to_string(),
          )
        })?;
      Ok(unsafe { objc2_core_foundation::CFRetained::from_raw(nn_ptr) })
    }
  }

  pub fn get_value<T: AXValueKind + AXValueDefault>(
    &self,
  ) -> crate::Result<T> {
    let mut value = T::ax_default();
    let success = unsafe {
      AXValueGetValue(
        self as *const Self as *mut accessibility_sys::__AXValue,
        T::TYPE,
        &mut value as *mut T as *mut c_void,
      )
    };

    if success {
      Ok(value)
    } else {
      Err(crate::Error::AXValueCreation(
        "AXValueGetValue failed".to_string(),
      ))
    }
  }
}

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

  pub fn AXUIElementSetAttributeValue(
    element: AXUIElementRef,
    attribute: CFStringRef,
    value: *const CFType,
  ) -> AXError;

  pub fn AXUIElementCreateApplication(pid: ProcessId) -> AXUIElementRef;

  pub fn AXObserverGetRunLoopSource(
    observer: AXObserverRef,
  ) -> CFRunLoopSourceRef;

  pub fn AXValueCreate(
    theType: AXValueType,
    valuePtr: *const c_void,
  ) -> AXValueRef;
}
