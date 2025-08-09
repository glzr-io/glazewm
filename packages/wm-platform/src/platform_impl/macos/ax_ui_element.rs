use std::{
  ops::Deref,
  ptr::{self, NonNull},
};

use anyhow::Result;
use objc2_core_foundation::{CFBoolean, CFRetained, CFString, CFType};

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
  fn get_attribute<T: FromAXValue>(&self, attribute: &str) -> Result<T>;
  fn set_attribute<T: IntoAXValue>(
    &self,
    attribute: &str,
    value: &T,
  ) -> Result<()>;
}

impl AXUIElementExt for CFRetained<AXUIElement> {
  fn get_attribute<T: FromAXValue>(&self, attribute: &str) -> Result<T> {
    let cf_attribute = CFString::from_str(attribute);
    let mut value: *mut CFType = ptr::null_mut();

    let result = unsafe {
      AXUIElementCopyAttributeValue(
        CFRetained::as_ptr(self).as_ptr(),
        cf_attribute.deref(),
        &mut value,
      )
    };

    if result != 0 {
      return Err(anyhow::anyhow!("AX get_attribute failed: {}", result));
    }

    if value.is_null() {
      return Err(anyhow::anyhow!("AX get_attribute returned null"));
    }

    unsafe { T::from_ax_value(value) }
  }

  fn set_attribute<T: IntoAXValue>(
    &self,
    attribute: &str,
    value: &T,
  ) -> Result<()> {
    let cf_attribute = CFString::from_str(attribute);
    let cf_value = value.into_ax_value()?;
    let result = unsafe {
      crate::platform_impl::AXUIElementSetAttributeValue(
        CFRetained::as_ptr(self).as_ptr(),
        cf_attribute.deref(),
        CFRetained::as_ptr(&cf_value).as_ptr().cast(),
      )
    };
    if result != 0 {
      return Err(anyhow::anyhow!("AX set_attribute failed: {}", result));
    }
    Ok(())
  }
}

pub trait FromAXValue: Sized {
  unsafe fn from_ax_value(raw: *mut CFType) -> Result<Self>;
}

pub trait IntoAXValue {
  fn into_ax_value(&self) -> Result<CFRetained<CFType>>;
}

impl FromAXValue for String {
  unsafe fn from_ax_value(raw: *mut CFType) -> Result<Self> {
    let any_value: CFRetained<CFType> = CFRetained::from_raw(
      std::ptr::NonNull::new_unchecked(raw as *mut CFType),
    );
    let cf_string = any_value
      .downcast::<CFString>()
      .map_err(|_| anyhow::anyhow!("AX value is not CFString"))?;
    Ok(cf_string.to_string())
  }
}

impl FromAXValue for bool {
  unsafe fn from_ax_value(raw: *mut CFType) -> Result<Self> {
    let any_value: CFRetained<CFType> = CFRetained::from_raw(
      std::ptr::NonNull::new_unchecked(raw as *mut CFType),
    );
    let cf_bool = any_value
      .downcast::<CFBoolean>()
      .map_err(|_| anyhow::anyhow!("AX value is not CFBoolean"))?;
    Ok(cf_bool.value())
  }
}

impl IntoAXValue for String {
  fn into_ax_value(&self) -> Result<CFRetained<CFType>> {
    let s = CFString::from_str(self);
    Ok(CFRetained::from(s))
  }
}

impl IntoAXValue for bool {
  fn into_ax_value(&self) -> Result<CFRetained<CFType>> {
    // CFBoolean singletons are not owned, but CF APIs typically accept
    // CFType. Represent by converting the static CFBoolean to CFType
    // and retaining.
    let cf = if *self {
      CFBoolean::new(true)
    } else {
      CFBoolean::new(false)
    };
    Ok(CFRetained::from(cf))
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
