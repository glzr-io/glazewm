use std::{
  fs::{self, File, TryLockError},
  path::PathBuf,
};

/// macOS-specific implementation of [`SingleInstance`].
pub(crate) struct SingleInstance {
  /// File that holds the lock.
  ///
  /// The lock is automatically released when the [`File`] is dropped.
  _file: File,
}

impl SingleInstance {
  /// macOS-specific implementation of [`SingleInstance::new`].
  pub(crate) fn new() -> crate::Result<Self> {
    let path = Self::lock_file_path()?;

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

  /// macOS-specific implementation of [`SingleInstance::is_running`].
  #[must_use]
  pub(crate) fn is_running() -> bool {
    let Ok(file) = Self::lock_file_path()
      .and_then(|path| File::open(&path).map_err(crate::Error::Io))
    else {
      return false;
    };

    // If `try_lock` fails with `WouldBlock`, the lock is held by another
    // process.
    file
      .try_lock()
      .is_err_and(|err| matches!(err, TryLockError::WouldBlock))
  }

  /// Returns the path to the lock file:
  /// `~/Library/Application Support/glazewm/.lock`.
  fn lock_file_path() -> crate::Result<PathBuf> {
    let home = home::home_dir().ok_or_else(|| {
      crate::Error::Platform(
        "Unable to determine home directory.".to_string(),
      )
    })?;

    Ok(home.join("Library/Application Support/glazewm/.lock"))
  }
}
