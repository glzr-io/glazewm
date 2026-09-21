/// Implements [`crate::session_id`].
pub(crate) fn session_id() -> u32 {
  // macOS has a single GUI session at a time.
  0
}
