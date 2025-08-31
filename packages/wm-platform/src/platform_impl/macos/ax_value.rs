use std::{ffi::c_void, mem::MaybeUninit, ptr::NonNull};

use objc2_application_services::{AXValue, AXValueType};
use objc2_core_foundation::{
  CFRange, CFRetained, CGPoint, CGRect, CGSize,
};

/// Trait for types that can be converted to and from `AXValue`.
pub trait AXValueTypeMarker: Sized + Copy {
  /// The `AXValueType` constant for this type.
  const AX_TYPE: AXValueType;
}

impl AXValueTypeMarker for CGPoint {
  const AX_TYPE: AXValueType = AXValueType::CGPoint;
}

impl AXValueTypeMarker for CGSize {
  const AX_TYPE: AXValueType = AXValueType::CGSize;
}

impl AXValueTypeMarker for CGRect {
  const AX_TYPE: AXValueType = AXValueType::CGRect;
}

impl AXValueTypeMarker for CFRange {
  const AX_TYPE: AXValueType = AXValueType::CFRange;
}

/// Extension trait for `AXValue`.
pub trait AXValueExt {
  /// Creates a new `AXValue` from the given value.
  ///
  /// This is a wrapper over the `AXValue::new` method from `objc2`.
  ///
  /// # Errors
  ///
  /// Returns an error if the `AXValue` creation fails.
  fn new_strict<T: AXValueTypeMarker>(
    val: &T,
  ) -> crate::Result<CFRetained<AXValue>>;

  /// Extracts the value from this `AXValue`.
  ///
  /// This is a wrapper over the `AXValue::value` method from `objc2`.
  ///
  /// # Errors
  ///
  /// Returns an error if:
  /// - The `AXValue` type doesn't match the requested type `T`.
  /// - The accessibility framework fails to extract the value.
  fn value_strict<T: AXValueTypeMarker>(&self) -> crate::Result<T>;
}

impl AXValueExt for AXValue {
  fn new_strict<T: AXValueTypeMarker>(
    val: &T,
  ) -> crate::Result<CFRetained<AXValue>> {
    let ptr = NonNull::new(std::ptr::from_ref::<T>(val) as *mut c_void)
      .ok_or_else(|| {
        crate::Error::InvalidPointer("Value pointer is null".to_string())
      })?;

    unsafe { AXValue::new(T::AX_TYPE, ptr) }.ok_or_else(|| {
      crate::Error::AXValueCreation(format!(
        "Failed to create AXValue for type with AX_TYPE {:?}",
        T::AX_TYPE
      ))
    })
  }

  fn value_strict<T: AXValueTypeMarker>(&self) -> crate::Result<T> {
    let mut value = MaybeUninit::<T>::uninit();
    let ptr = NonNull::new(value.as_mut_ptr().cast::<c_void>())
      .ok_or_else(|| {
        crate::Error::InvalidPointer(
          "Value buffer pointer is null".to_string(),
        )
      })?;

    let success = unsafe { self.value(T::AX_TYPE, ptr) };

    if success {
      Ok(unsafe { value.assume_init() })
    } else {
      Err(crate::Error::AXValueCreation(format!(
        "Failed to extract value from AXValue for type with AX_TYPE {:?}",
        T::AX_TYPE
      )))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_ax_value_creation_and_extraction() {
    let point = CGPoint { x: 10.0, y: 20.0 };
    let ax_value =
      AXValue::new_strict(&point).expect("Failed to create AXValue.");

    let extracted_point: CGPoint = ax_value
      .value_strict()
      .expect("Failed to extract value from AXValue.");

    assert!((point.x - extracted_point.x).abs() < f64::EPSILON);
    assert!((point.y - extracted_point.y).abs() < f64::EPSILON);
  }

  #[test]
  fn test_ax_value_wrong_type_extraction() {
    let point = CGPoint { x: 10.0, y: 20.0 };
    let ax_value =
      AXValue::new_strict(&point).expect("Failed to create AXValue.");

    // Try to extract as `CGSize` instead of `CGPoint`.
    let result = ax_value.value_strict::<CGSize>();
    assert!(result.is_err());
  }
}
