use std::{ffi::c_void, mem::MaybeUninit, ptr::NonNull};

use accessibility_sys::{
  kAXValueTypeCFRange, kAXValueTypeCGPoint, kAXValueTypeCGRect,
  kAXValueTypeCGSize, AXError,
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

/// Trait for types that can be converted to and from `AXValue`.
pub trait AXValueConvertible: Sized + Copy {
  /// The `AXValueType` constant for this type.
  const AX_TYPE: AXValueType;
}

impl AXValueConvertible for CGPoint {
  const AX_TYPE: AXValueType = kAXValueTypeCGPoint;
}

impl AXValueConvertible for CGSize {
  const AX_TYPE: AXValueType = kAXValueTypeCGSize;
}

impl AXValueConvertible for CGRect {
  const AX_TYPE: AXValueType = kAXValueTypeCGRect;
}

impl AXValueConvertible for CFRange {
  const AX_TYPE: AXValueType = kAXValueTypeCFRange;
}

impl AXValue {
  /// Creates a new `AXValue` from the given value.
  ///
  /// # Errors
  ///
  /// Returns an error if the `AXValue` creation fails.
  pub fn new<T: AXValueConvertible>(
    val: &T,
  ) -> crate::Result<objc2_core_foundation::CFRetained<Self>> {
    let ptr = unsafe {
      AXValueCreate(T::AX_TYPE, val as *const T as *const c_void)
    };

    if ptr.is_null() {
      Err(crate::Error::AXValueCreation(format!(
        "Failed to create AXValue for type with AX_TYPE {}",
        T::AX_TYPE
      )))
    } else {
      let nn_ptr = NonNull::new(ptr as *mut Self).ok_or_else(|| {
        crate::Error::AXValueCreation(
          "AXValueCreate returned non-null but pointer conversion failed"
            .to_string(),
        )
      })?;

      // SAFETY: Pointer is verified to be non-null.
      Ok(unsafe { objc2_core_foundation::CFRetained::from_raw(nn_ptr) })
    }
  }

  /// Extracts the value from this `AXValue`.
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - The `AXValue` type doesn't match the requested type `T`.
  /// - The accessibility framework fails to extract the value.
  pub fn value<T: AXValueConvertible>(&self) -> crate::Result<T> {
    let mut value = MaybeUninit::<T>::uninit();
    let success = unsafe {
      AXValueGetValue(
        self as *const Self as *mut Self,
        T::AX_TYPE,
        value.as_mut_ptr() as *mut c_void,
      )
    };

    if success {
      Ok(unsafe { value.assume_init() })
    } else {
      Err(crate::Error::AXValueCreation(format!(
        "Failed to extract value from AXValue for type with AX_TYPE {}",
        T::AX_TYPE
      )))
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

  pub fn AXValueGetValue(
    value: AXValueRef,
    theType: AXValueType,
    valuePtr: *mut c_void,
  ) -> bool;
}
