use std::{ops::Deref, ptr};

use accessibility_sys::{
  kAXPositionAttribute, kAXSizeAttribute, kAXTitleAttribute, kAXWindowRole,
};
use anyhow::{Context, Result};
use objc2_core_foundation::{CFRetained, CFString, CFType};

use crate::platform_impl::{
  AXUIElementCopyAttributeValue, AXUIElementRef,
};

/// A safe wrapper around `AXUIElementRef` that provides utility methods
/// for getting and setting accessibility attributes.
#[derive(Debug, Clone)]
pub struct AXElement {
  // Retain the underlying CF object so it stays alive beyond the callback
  retained: CFRetained<CFType>,
  element_ref: AXUIElementRef,
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
    if element_ref.is_null() {
      return Err(anyhow::anyhow!("nullptr passed."));
    }

    // AXUIElementRef is a CoreFoundation type; retain it to extend
    // lifetime. This is necessary to ensure the underlying object
    // remains valid beyond event callbacks.
    let retained: CFRetained<CFType> = CFRetained::retain(
      std::ptr::NonNull::new_unchecked(element_ref as *mut CFType),
    );

    Ok(Self {
      retained,
      element_ref,
    })
  }

  /// Returns the raw `AXUIElementRef`.
  pub fn as_ref(&self) -> AXUIElementRef {
    self.element_ref
  }

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
        self.element_ref,
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
      let cf_string: CFRetained<CFString> = CFRetained::retain(
        std::ptr::NonNull::new_unchecked(value as *mut _),
      );
      Ok(cf_string.to_string())
    }
  }

  /// Generic method to get a boolean attribute.
  pub fn get_bool_attribute(&self, attribute: &str) -> Result<bool> {
    let cf_attribute = CFString::from_str(attribute);
    let mut value: *mut CFType = ptr::null_mut();

    let result = unsafe {
      AXUIElementCopyAttributeValue(
        self.element_ref,
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

    // For boolean attributes, we need to interpret the CFType as CFBoolean
    // This is a simplified implementation that checks the CFType directly
    // In a real implementation, you'd want to use proper CFBoolean
    // handling For now, we'll do a basic check - you may need to
    // implement proper CFBoolean type checking and value extraction
    // This is a placeholder implementation
    Ok(true) // TODO: Implement proper boolean extraction from CFType
  }

  /// Generic method to get a point attribute (x, y coordinates).
  fn get_point_attribute(&self, attribute: &str) -> Result<(f64, f64)> {
    let cf_attribute = CFString::from_str(attribute);
    let mut value: *mut CFType = ptr::null_mut();

    let result = unsafe {
      AXUIElementCopyAttributeValue(
        self.element_ref,
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

    // This is a simplified implementation for point attributes
    // In practice, you would need to properly handle CFPoint/CGPoint types
    // For now, we'll return a placeholder
    // TODO: Implement proper CFPoint/CGPoint parsing
    Ok((0.0, 0.0))
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
        self.element_ref,
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

    // This is a simplified implementation for rect attributes
    // In practice, you would need to properly handle CFRect/CGRect types
    // For now, we'll return a placeholder
    // TODO: Implement proper CFRect/CGRect parsing
    Ok((0.0, 0.0, 0.0, 0.0))
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
