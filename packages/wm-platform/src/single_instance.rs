use crate::platform_impl;

/// Ensures only one instance of the application is running at a time.
///
/// # Platform-specific
///
/// - **Windows**: Uses a named system-wide mutex.
/// - **macOS**: Uses an exclusive file lock.
pub struct SingleInstance {
  /// Inner platform-specific single instance implementation.
  _inner: platform_impl::SingleInstance,
}

impl SingleInstance {
  /// Creates a new [`SingleInstance`], acquiring the platform-specific
  /// lock or mutex.
  ///
  /// # Errors
  ///
  /// Returns [`Error::Platform`] if another instance is already running.
  pub fn new() -> crate::Result<Self> {
    let inner = platform_impl::SingleInstance::new()?;
    Ok(Self { _inner: inner })
  }

  /// Returns whether another instance of the application is currently
  /// running.
  #[must_use]
  pub fn is_running() -> bool {
    platform_impl::SingleInstance::is_running()
  }
}
