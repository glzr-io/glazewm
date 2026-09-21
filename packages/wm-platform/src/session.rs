use crate::platform_impl;

/// ID of the OS session that the current process belongs to.
///
/// Used to keep per-session files apart, so that one instance of the WM
/// can run for each logged-in user.
///
/// # Platform-specific
///
/// - **Windows**: Terminal Services session ID.
/// - **macOS**: Always `0`, since there is a single GUI session at a time.
#[must_use]
pub fn session_id() -> u32 {
  platform_impl::session_id()
}
