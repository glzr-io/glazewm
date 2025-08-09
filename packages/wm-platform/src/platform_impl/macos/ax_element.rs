use std::{
  fmt,
  ops::Deref,
  ptr::{self, NonNull},
};

use accessibility_sys::{
  kAXPositionAttribute, kAXSizeAttribute, kAXTitleAttribute, kAXWindowRole,
};
use anyhow::{Context, Result};
use objc2_core_foundation::{CFBoolean, CFRetained, CFString, CFType};

use crate::platform_impl::{
  AXUIElement, AXUIElementCopyAttributeValue, AXUIElementRef,
};

/// A safe wrapper around `AXUIElementRef` that provides utility methods
/// for getting and setting accessibility attributes.
pub struct AXElement {
  // Retain the underlying AXUIElement so it stays alive beyond the
  // callback
  element_ref: CFRetained<AXUIElement>,
}

impl fmt::Debug for AXElement {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "AXElement({:p})",
      CFRetained::as_ptr(&self.element_ref).as_ptr()
    )
  }
}

impl AXElement {
  /// Creates a new `AXElement` from an `AXUIElementRef`.
  ///
  /// # Safety
  /// The caller must ensure that `element_ref` is a valid, non-null
  /// `AXUIElementRef`.
  pub unsafe fn from_ref(
    element_ref: AXUIElementRef,
  ) -> anyhow::Result<Self> {
    let ptr = NonNull::new(element_ref)
      .map(std::ptr::NonNull::cast)
      .ok_or(anyhow::anyhow!("nullptr passed."))?;

    // Retain the AXUIElement itself
    let retained: CFRetained<AXUIElement> = CFRetained::retain(ptr);

    Ok(Self {
      element_ref: retained,
    })
  }

  // /// Returns the raw `AXUIElementRef`.
  // pub fn as_ref(&self) -> AXUIElementRef {
  //   self.element_ref
  // }

  /// Gets the title of the element (typically used for windows).
  pub fn title(&self) -> anyhow::Result<String> {
    self
      .get_string_attribute(kAXTitleAttribute)
      .with_context(|| "Failed to get title attribute")
  }

  /// Gets the role of the element (e.g., window, button, etc.).
  pub fn role(&self) -> Result<String> {
    self
      .get_string_attribute(kAXWindowRole)
      .with_context(|| "Failed to get role attribute")
  }

  /// Checks if the element is minimized (for windows).
  pub fn is_minimized(&self) -> Result<bool> {
    // Note: kAXIsMinimizedAttribute may not be available in
    // accessibility_sys You can use the string constant directly
    self
      .get_bool_attribute("AXMinimized")
      .with_context(|| "Failed to get minimized state")
  }

  /// Gets the position of the element as (x, y) coordinates.
  pub fn position(&self) -> Result<(f64, f64)> {
    self
      .get_point_attribute(kAXPositionAttribute)
      .with_context(|| "Failed to get position attribute")
  }

  /// Gets the size of the element as (width, height).
  pub fn size(&self) -> Result<(f64, f64)> {
    self
      .get_point_attribute(kAXSizeAttribute)
      .with_context(|| "Failed to get size attribute")
  }

  /// Gets the frame of the element as (x, y, width, height).
  pub fn frame(&self) -> Result<(f64, f64, f64, f64)> {
    // Try to get the frame attribute first, which should contain all the
    // info
    match self.get_rect_attribute("AXFrame") {
      Ok(frame) => Ok(frame),
      Err(_) => {
        // Fall back to getting position and size separately
        let (x, y) = self.position()?;
        let (width, height) = self.size()?;
        Ok((x, y, width, height))
      }
    }
  }

  /// Generic method to get a string attribute.
  pub fn get_string_attribute(&self, attribute: &str) -> Result<String> {
    let cf_attribute = CFString::from_str(attribute);
    let mut value: *mut CFType = ptr::null_mut();

    let result = unsafe {
      AXUIElementCopyAttributeValue(
        CFRetained::as_ptr(&self.element_ref).as_ptr(),
        cf_attribute.deref(),
        &mut value,
      )
    };

    if result != 0 {
      return Err(anyhow::anyhow!(
        "Failed to get attribute '{}': error code {}",
        attribute,
        result
      ));
    }

    if value.is_null() {
      return Err(anyhow::anyhow!(
        "Attribute '{}' returned null value",
        attribute
      ));
    }

    unsafe {
      let any_value: CFRetained<CFType> = CFRetained::from_raw(
        std::ptr::NonNull::new_unchecked(value as *mut CFType),
      );
      match any_value.downcast::<CFString>() {
        Ok(cf_string) => Ok(cf_string.to_string()),
        Err(_) => Err(anyhow::anyhow!(
          "Attribute '{}' is not a CFString",
          attribute
        )),
      }
    }
  }

  /// Generic method to get a boolean attribute.
  pub fn get_bool_attribute(&self, attribute: &str) -> Result<bool> {
    let cf_attribute = CFString::from_str(attribute);
    let mut value: *mut CFType = ptr::null_mut();

    let result = unsafe {
      AXUIElementCopyAttributeValue(
        CFRetained::as_ptr(&self.element_ref).as_ptr(),
        cf_attribute.deref(),
        &mut value,
      )
    };

    if result != 0 {
      return Err(anyhow::anyhow!(
        "Failed to get attribute '{}': error code {}",
        attribute,
        result
      ));
    }

    if value.is_null() {
      return Err(anyhow::anyhow!(
        "Attribute '{}' returned null value",
        attribute
      ));
    }

    // Interpret the CFType as CFBoolean
    unsafe {
      let any_value: CFRetained<CFType> = CFRetained::from_raw(
        std::ptr::NonNull::new_unchecked(value as *mut CFType),
      );
      match any_value.downcast::<CFBoolean>() {
        Ok(cf_bool) => Ok(cf_bool.value()),
        Err(_) => Err(anyhow::anyhow!(
          "Attribute '{}' is not a CFBoolean",
          attribute
        )),
      }
    }
  }

  /// Generic method to get a point attribute (x, y coordinates).
  fn get_point_attribute(&self, attribute: &str) -> Result<(f64, f64)> {
    let cf_attribute = CFString::from_str(attribute);
    let mut value: *mut CFType = ptr::null_mut();

    let result = unsafe {
      AXUIElementCopyAttributeValue(
        CFRetained::as_ptr(&self.element_ref).as_ptr(),
        cf_attribute.deref(),
        &mut value,
      )
    };

    if result != 0 {
      return Err(anyhow::anyhow!(
        "Failed to get attribute '{}': error code {}",
        attribute,
        result
      ));
    }

    if value.is_null() {
      return Err(anyhow::anyhow!(
        "Attribute '{}' returned null value",
        attribute
      ));
    }

    // TODO: Implement AXValue (CGPoint) extraction
    Err(anyhow::anyhow!(
      "Attribute '{}' point parsing not implemented",
      attribute
    ))
  }

  /// Generic method to get a rect attribute (x, y, width, height).
  fn get_rect_attribute(
    &self,
    attribute: &str,
  ) -> Result<(f64, f64, f64, f64)> {
    let cf_attribute = CFString::from_str(attribute);
    let mut value: *mut CFType = ptr::null_mut();

    let result = unsafe {
      AXUIElementCopyAttributeValue(
        CFRetained::as_ptr(&self.element_ref).as_ptr(),
        cf_attribute.deref(),
        &mut value,
      )
    };

    if result != 0 {
      return Err(anyhow::anyhow!(
        "Failed to get attribute '{}': error code {}",
        attribute,
        result
      ));
    }

    if value.is_null() {
      return Err(anyhow::anyhow!(
        "Attribute '{}' returned null value",
        attribute
      ));
    }

    // TODO: Implement AXValue (CGRect) extraction
    Err(anyhow::anyhow!(
      "Attribute '{}' rect parsing not implemented",
      attribute
    ))
  }
}

unsafe impl Send for AXElement {}
unsafe impl Sync for AXElement {}

// No Drop implementation; allow Copy semantics for this raw-pointer
// wrapper

#[cfg(test)]
mod tests {
  use super::*;

  // Note: These tests require a real macOS environment with accessibility
  // permissions to run properly.

  #[test]
  fn test_ax_element_creation() {
    // This test would need a real AXUIElementRef to be meaningful
    // For now, just test that the struct can be created
    let null_ref = std::ptr::null_mut();
    let element = unsafe { AXElement::from_ref(null_ref) };
    assert!(element.is_err());
  }
}
