use std::{
  fs::{self, File, TryLockError},
  path::PathBuf,
};

/// Ensures only one instance of the application is running.
///
/// Uses a file lock on
/// `~/Library/Application Support/glzr.io/glazewm.lock` via
/// `File::try_lock()`.
pub struct SingleInstance {
  /// File that holds the lock.
  ///
  /// The lock is automatically released when the `File` is dropped.
  _file: File,
}

impl SingleInstance {
  /// Creates a new `SingleInstance` by acquiring an exclusive lock
  /// on the lock file.
  ///
  /// Returns a `Platform` error if another instance already holds
  /// the lock.
  pub fn new() -> crate::Result<Self> {
    let path = lock_file_path()?;

    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent).map_err(crate::Error::Io)?;
    }

    let file = File::create(&path).map_err(crate::Error::Io)?;

    // Acquire exclusive file lock.
    file.try_lock().map_err(|err| match err {
      TryLockError::WouldBlock => crate::Error::Platform(
        "Another instance of the application is already running."
          .to_string(),
      ),
      TryLockError::Error(io_err) => crate::Error::Io(io_err),
    })?;

    Ok(Self { _file: file })
  }

  /// Returns whether another instance of the application is currently
  /// running.
  #[must_use]
  pub fn is_running() -> bool {
    let Ok(path) = lock_file_path() else {
      return false;
    };

    let Ok(file) = File::open(&path) else {
      return false;
    };

    // If `try_lock` fails with `WouldBlock`, the lock is held by another
    // process.
    file
      .try_lock()
      .is_err_and(|err| matches!(err, TryLockError::WouldBlock))
  }
}

/// Returns the path to the lock file:
/// `~/Library/Application Support/glzr.io/glazewm.lock`.
fn lock_file_path() -> crate::Result<PathBuf> {
  let home = home::home_dir().ok_or_else(|| {
    crate::Error::Platform(
      "Unable to determine home directory.".to_string(),
    )
  })?;

  Ok(home.join("Library/Application Support/glzr.io/glazewm.lock"))
}
