use std::ptr::{self, NonNull};

pub use objc2_application_services::{AXError, AXUIElement};
use objc2_core_foundation::{CFRetained, CFString, CFType};

use crate::Error;

pub type AXUIElementRef = *mut AXUIElement;

/// Extension trait for `AXUIElement`.
pub trait AXUIElementExt {
  /// Creates a retained `AXUIElement` from a raw `AXUIElementRef`.
  ///
  /// # Errors
  ///
  /// Returns an error if `element_ref` is a null pointer.
  fn from_ref(
    element_ref: AXUIElementRef,
  ) -> crate::Result<CFRetained<AXUIElement>>;

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
  fn set_attribute<T: objc2_core_foundation::Type + AsRef<CFType>>(
    &self,
    attribute: &str,
    value: &CFRetained<T>,
  ) -> crate::Result<()>;
}

impl AXUIElementExt for AXUIElement {
  fn from_ref(
    element_ref: AXUIElementRef,
  ) -> crate::Result<CFRetained<Self>> {
    let ptr = NonNull::new(element_ref).ok_or_else(|| {
      Error::InvalidPointer("AXUIElementRef is null".to_string())
    })?;

    // SAFETY: Pointer is verified to be non-null.
    Ok(unsafe { CFRetained::retain(ptr.cast()) })
  }

  fn get_attribute<T: objc2_core_foundation::Type>(
    &self,
    attribute: &str,
  ) -> crate::Result<CFRetained<T>> {
    let cf_attribute = CFString::from_str(attribute);
    let mut value: *const CFType = ptr::null();

    let result = unsafe {
      self.copy_attribute_value(
        &cf_attribute,
        NonNull::new(&raw mut value).expect("Failed to get mut ptr"),
      )
    };

    if result != AXError::Success {
      return Err(Error::Accessibility(result.0));
    }

    NonNull::new(value.cast_mut())
      .map(|ptr| {
        // SAFETY: The copy_attribute_value function guarantees that
        // if it succeeds (result == 0), the returned pointer is valid and
        // properly initialized. We take ownership of this retained
        // reference.
        unsafe { CFRetained::from_raw(ptr.cast()) }
      })
      .ok_or_else(|| {
        Error::InvalidPointer(
          "copy_attribute_value returned success but null pointer"
            .to_string(),
        )
      })
  }

  fn set_attribute<T: objc2_core_foundation::Type + AsRef<CFType>>(
    &self,
    attribute: &str,
    value: &CFRetained<T>,
  ) -> crate::Result<()> {
    let cf_attribute = CFString::from_str(attribute);
    let result =
      unsafe { self.set_attribute_value(&cf_attribute, value.as_ref()) };

    if result != AXError::Success {
      return Err(Error::Accessibility(result.0));
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
