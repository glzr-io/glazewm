use anyhow::{bail, Context, Result};
use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{
      CloseHandle, GetLastError, ERROR_ALREADY_EXISTS,
      ERROR_FILE_NOT_FOUND, HANDLE,
    },
    System::Threading::{
      CreateMutexW, OpenMutexW, ReleaseMutex,
      SYNCHRONIZATION_ACCESS_RIGHTS,
    },
  },
};

pub struct SingleInstance {
  handle: HANDLE,
}

/// Arbitrary GUID used to identify the application.
const APP_GUID: PCWSTR =
  w!("Global\\325d0ed7-7f60-4925-8d1b-aa287b26b218");

impl SingleInstance {
  /// Creates a new system-wide mutex to ensure that only one instance of
  /// the application is running.
  pub fn new() -> Result<Self> {
    let handle = unsafe { CreateMutexW(None, true, APP_GUID) }
      .context("Failed to create single instance mutex.")?;

    if let Err(err) = unsafe { GetLastError() } {
      if err == ERROR_ALREADY_EXISTS.into() {
        bail!("Another instance of the application is already running.");
      }
    }

    Ok(Self { handle })
  }

  /// Gets whether there is an active instance of the application.
  #[must_use]
  pub fn is_running() -> bool {
    let res = unsafe {
      OpenMutexW(SYNCHRONIZATION_ACCESS_RIGHTS::default(), false, APP_GUID)
    };

    // Check whether the mutex exists. If it doesn't, then this is the
    // only instance.
    match res {
      Ok(_) => false,
      Err(err) => err == ERROR_FILE_NOT_FOUND.into(),
    }
  }
}

impl Drop for SingleInstance {
  fn drop(&mut self) {
    unsafe {
      let _ = ReleaseMutex(self.handle);
      let _ = CloseHandle(self.handle);
    }
  }
}
