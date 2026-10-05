//! Utilities for testing.
//!
//! Available via the `test_utils` Cargo feature.
use std::sync::{atomic::AtomicBool, Arc};

use crate::platform_impl;
pub use crate::{Dispatcher, Display, NativeWindow};

impl Dispatcher {
  /// Creates a mock `Dispatcher` for use in tests.
  ///
  /// Calling any methods on the mock is undefined behavior and may panic.
  #[must_use]
  pub fn mock() -> Self {
    Self::new(None, Arc::new(AtomicBool::new(false)))
  }
}

impl NativeWindow {
  /// Creates a mock `NativeWindow` for use in tests.
  ///
  /// Calling any methods on the mock is undefined behavior and may panic.
  #[must_use]
  pub fn mock() -> Self {
    platform_impl::NativeWindow::new(0).into()
  }
}

impl Display {
  /// Creates a mock `Display` for use in tests.
  ///
  /// Calling any methods on the mock is undefined behavior and may panic.
  #[must_use]
  pub fn mock() -> Self {
    Self {
      inner: platform_impl::Display::new(0),
    }
  }
}
