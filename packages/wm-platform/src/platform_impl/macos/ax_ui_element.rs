use std::ptr::{self, NonNull};

use objc2_core_foundation::{CFRetained, CFString, CFType};

use crate::{
  platform_impl::{
    AXUIElementCopyAttributeValue, AXUIElementSetAttributeValue,
  },
  Error,
};

const AX_ERROR_SUCCESS: i32 = 0;

/// Opaque CF type representing `AXUIElement`.
///
/// It follows `CFRetain` / `CFRelease` semantics.
#[derive(Debug)]
pub struct AXUIElement;
pub type AXUIElementRef = *mut AXUIElement;

// Mark the opaque CF type so that it can be used with `CFRetained`.
unsafe impl objc2_core_foundation::Type for AXUIElement {}

impl AXUIElement {
  /// Creates a retained `AXUIElement` from a raw `AXUIElementRef`.
  ///
  /// # Errors
  ///
  /// Returns an error if `element_ref` is a null pointer.
  pub fn from_ref(
    element_ref: AXUIElementRef,
  ) -> crate::Result<CFRetained<Self>> {
    let ptr =
      NonNull::new(element_ref)
        .map(NonNull::cast)
        .ok_or_else(|| {
          Error::InvalidPointer("AXUIElementRef is null".to_string())
        })?;

    // SAFETY: Pointer is verified to be non-null.
    Ok(unsafe { CFRetained::retain(ptr) })
  }
}

/// Extension trait for `AXUIElement`.
pub trait AXUIElementExt {
  /// Retrieves the value of an accessibility attribute.
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - The accessibility operation fails (e.g., invalid attribute name).
  /// - The attribute value cannot be cast to the requested type.
  fn get_attribute<T: objc2_core_foundation::Type>(
    &self,
    attribute: &str,
  ) -> crate::Result<CFRetained<T>>;

  /// Sets the value of an accessibility attribute.
  ///
  /// # Errors
  ///
  /// Returns an error if the accessibility operation fails.
  fn set_attribute<T: objc2_core_foundation::Type>(
    &self,
    attribute: &str,
    value: &CFRetained<T>,
  ) -> crate::Result<()>;
}

impl AXUIElementExt for CFRetained<AXUIElement> {
  fn get_attribute<T: objc2_core_foundation::Type>(
    &self,
    attribute: &str,
  ) -> crate::Result<CFRetained<T>> {
    let cf_attribute = CFString::from_str(attribute);
    let mut value: *mut CFType = ptr::null_mut();

    let result = unsafe {
      AXUIElementCopyAttributeValue(
        CFRetained::as_ptr(self).as_ptr(),
        &raw const *cf_attribute,
        &mut value,
      )
    };

    if result != AX_ERROR_SUCCESS {
      return Err(Error::Accessibility(result));
    }

    NonNull::new(value)
      .map(|ptr| {
        // SAFETY: The AXUIElementCopyAttributeValue function guarantees that
        // if it succeeds (result == 0), the returned pointer is valid and
        // properly initialized. We take ownership of this retained reference.
        unsafe { CFRetained::from_raw(ptr.cast()) }
      })
      .ok_or_else(|| {
        Error::InvalidPointer(
          "AXUIElementCopyAttributeValue returned success but null pointer".to_string(),
        )
      })
  }

  fn set_attribute<T: objc2_core_foundation::Type>(
    &self,
    attribute: &str,
    value: &CFRetained<T>,
  ) -> crate::Result<()> {
    let cf_attribute = CFString::from_str(attribute);
    let result = unsafe {
      AXUIElementSetAttributeValue(
        CFRetained::as_ptr(self).as_ptr(),
        &raw const *cf_attribute,
        CFRetained::as_ptr(value).as_ptr() as *const CFType,
      )
    };

    if result != AX_ERROR_SUCCESS {
      return Err(Error::Accessibility(result));
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ax_element_creation_with_null_pointer() {
    let null_ref = std::ptr::null_mut();
    let result = AXUIElement::from_ref(null_ref);

    assert!(matches!(result, Err(Error::InvalidPointer(_))));
  }
}
