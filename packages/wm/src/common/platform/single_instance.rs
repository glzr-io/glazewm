use anyhow::{Context, Result};
use windows::{
  core::{w, PCWSTR},
  Win32::{
    Foundation::{
      CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE,
    },
    System::Threading::{CreateMutexW, ReleaseMutex},
  },
};

pub struct SingleInstance {
  handle: Option<HANDLE>,
}

const APP_GUID: PCWSTR =
  w!("Global\\325d0ed7-7f60-4925-8d1b-aa287b26b218");

impl SingleInstance {
  /// Creates a new instance of `SingleInstance` struct.
  pub fn new() -> Result<Self> {
    let handle = unsafe { CreateMutexW(None, true, APP_GUID) }
      .context("Failed to create single instance mutex.")?;

    if let Err(error) = unsafe { GetLastError() } {
      if error == ERROR_ALREADY_EXISTS.into() {
        // Another instance of the application is already running.
        return Ok(Self { handle: None });
      }
    }

    Ok(Self {
      handle: Some(handle),
    })
  }

  /// Gets whether this is the only active instance of the application.
  pub fn is_single(&self) -> bool {
    self.handle.is_some()
  }
}

impl Drop for SingleInstance {
  fn drop(&mut self) {
    if let Some(handle) = self.handle.take() {
      unsafe {
        let _ = ReleaseMutex(handle);
        let _ = CloseHandle(handle);
      }
    }
  }
}
