/// Gets whether the current Windows version is Windows 11 or newer.
///
/// # Platform-specific
///
/// This method is only available on Windows.
pub fn is_windows_11_or_greater() -> crate::Result<bool> {
  crate::platform_impl::is_windows_11_or_greater()
}
