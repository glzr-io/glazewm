use std::ptr::{self, NonNull};

use objc2_core_foundation::{CFRetained, CFString, CFType};

use crate::platform_impl::AXUIElementCopyAttributeValue;

// Opaque CoreFoundation type representing AXUIElement
// AXUIElement is a CFType, not an Objective-C class.
// It follows CFRetain/CFRelease semantics.
#[derive(Debug)]
pub struct AXUIElement;
pub type AXUIElementRef = *mut AXUIElement;

// Mark the opaque CF type so it can be used with CFRetained
unsafe impl objc2_core_foundation::Type for AXUIElement {}

impl AXUIElement {
  /// Creates a retained AXUIElement from a raw `AXUIElementRef`.
  ///
  /// # Safety
  /// Caller must ensure the pointer is valid.
  pub unsafe fn from_ref(
    element_ref: AXUIElementRef,
  ) -> anyhow::Result<CFRetained<Self>> {
    let ptr = NonNull::new(element_ref)
      .map(std::ptr::NonNull::cast)
      .ok_or(anyhow::anyhow!("nullptr passed."))?;
    Ok(CFRetained::retain(ptr))
  }
}

pub trait AXUIElementExt {
  fn get_attribute<T: objc2_core_foundation::Type>(
    &self,
    attribute: &str,
  ) -> anyhow::Result<CFRetained<T>>;
  fn set_attribute<T: objc2_core_foundation::Type>(
    &self,
    attribute: &str,
    value: &CFRetained<T>,
  ) -> anyhow::Result<()>;
}

impl AXUIElementExt for CFRetained<AXUIElement> {
  fn get_attribute<T: objc2_core_foundation::Type>(
    &self,
    attribute: &str,
  ) -> anyhow::Result<CFRetained<T>> {
    let cf_attribute = CFString::from_str(attribute);
    let mut value: *mut CFType = ptr::null_mut();

    let result = unsafe {
      AXUIElementCopyAttributeValue(
        CFRetained::as_ptr(self).as_ptr(),
        &raw const *cf_attribute,
        &mut value,
      )
    };

    if result != 0 {
      return Err(anyhow::anyhow!("AX get_attribute failed: {}", result));
    }

    NonNull::new(value)
      .map(|ptr| unsafe { CFRetained::from_raw(ptr.cast()) })
      .ok_or(anyhow::anyhow!("AX get_attribute returned null."))
  }

  fn set_attribute<T: objc2_core_foundation::Type>(
    &self,
    attribute: &str,
    value: &CFRetained<T>,
  ) -> anyhow::Result<()> {
    let cf_attribute = CFString::from_str(attribute);
    let result = unsafe {
      crate::platform_impl::AXUIElementSetAttributeValue(
        CFRetained::as_ptr(self).as_ptr(),
        &raw const *cf_attribute,
        CFRetained::as_ptr(value).as_ptr() as *const CFType,
      )
    };

    if result != 0 {
      return Err(anyhow::anyhow!("AX set_attribute failed: {}", result));
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ax_element_creation() {
    let null_ref = std::ptr::null_mut();
    let element = unsafe { AXUIElement::from_ref(null_ref) };
    assert!(element.is_err());
  }
}
