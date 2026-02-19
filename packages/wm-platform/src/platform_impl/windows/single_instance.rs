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

/// Windows-specific implementation of [`SingleInstance`].
pub struct SingleInstance {
  handle: HANDLE,
}

/// Arbitrary GUID to uniquely identify the application.
const APP_GUID: PCWSTR =
  w!("Global\\325d0ed7-7f60-4925-8d1b-aa287b26b218");

impl SingleInstance {
  /// Windows-specific implementation of [`SingleInstance::new`].
  pub(crate) fn new() -> crate::Result<Self> {
    // Create a named system-wide mutex.
    let handle = unsafe { CreateMutexW(None, true, APP_GUID) }?;

    if let Err(err) = unsafe { GetLastError() } {
      if err == ERROR_ALREADY_EXISTS.into() {
        return Err(crate::Error::Platform(
          "Another instance of the application is already running."
            .to_string(),
        ));
      }
    }

    Ok(Self { handle })
  }

  /// Windows-specific implementation of [`SingleInstance::is_running`].
  #[must_use]
  pub(crate) fn is_running() -> bool {
    let res = unsafe {
      OpenMutexW(SYNCHRONIZATION_ACCESS_RIGHTS::default(), false, APP_GUID)
    };

    // If the mutex exists, then another instance is running.
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
