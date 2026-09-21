use windows::Win32::System::{
  RemoteDesktop::ProcessIdToSessionId, Threading::GetCurrentProcessId,
};

/// Implements [`crate::session_id`].
pub(crate) fn session_id() -> u32 {
  let mut session_id = 0;

  // Falling back to 0 is harmless: the ID only keeps per-session files
  // apart, and the lookup cannot fail for the current process.
  match unsafe {
    ProcessIdToSessionId(GetCurrentProcessId(), &raw mut session_id)
  } {
    Ok(()) => session_id,
    Err(_) => 0,
  }
}
