use std::path::Path;

use crate::platform_impl;

/// Registration of an application to start when the user signs in.
///
/// Backed by a scheduled task with a logon trigger rather than the `Run`
/// registry key, which Explorer only works through well after sign-in.
pub struct Autostart {
  /// Inner platform-specific autostart implementation.
  inner: platform_impl::Autostart,
}

impl Autostart {
  /// Creates an [`Autostart`] for the current user.
  ///
  /// `app_name` identifies the registration, and `exe_path` is the
  /// executable to start.
  ///
  /// # Errors
  ///
  /// Returns [`crate::Error::Platform`] if the current user cannot be
  /// resolved.
  pub fn new(app_name: &str, exe_path: &Path) -> crate::Result<Self> {
    let inner = platform_impl::Autostart::new(app_name, exe_path)?;
    Ok(Self { inner })
  }

  /// Returns whether the application is registered to start at sign-in.
  pub fn is_enabled(&self) -> crate::Result<bool> {
    self.inner.is_enabled()
  }

  /// Registers the application to start at sign-in.
  ///
  /// Replaces any existing registration, so this also repairs one that
  /// points at an outdated executable path.
  pub fn enable(&self) -> crate::Result<()> {
    self.inner.enable()
  }

  /// Removes the registration. Succeeds if there is none.
  pub fn disable(&self) -> crate::Result<()> {
    self.inner.disable()
  }

  /// Carries a registration made by an earlier version, under the `Run`
  /// registry key, over to the current mechanism.
  ///
  /// Returns whether such a registration was found.
  pub fn migrate_from_run_key(&self) -> crate::Result<bool> {
    self.inner.migrate_from_run_key()
  }
}
