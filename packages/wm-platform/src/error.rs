#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),

  #[error(transparent)]
  #[cfg(target_os = "windows")]
  Windows(#[from] windows::core::Error),

  #[error("Accessibility operation failed with error code: {0}")]
  #[cfg(target_os = "macos")]
  Accessibility(i32),

  #[error("Invalid pointer: {0}")]
  InvalidPointer(String),

  #[error("AXValue creation failed: {0}")]
  #[cfg(target_os = "macos")]
  AXValueCreation(String),

  #[error(transparent)]
  ChannelRecv(#[from] std::sync::mpsc::RecvError),

  #[error(transparent)]
  Anyhow(#[from] anyhow::Error),

  #[error("Display enumeration failed")]
  DisplayEnumerationFailed,

  #[error("Display mode not found")]
  DisplayModeNotFound,

  #[error("Primary display not found")]
  PrimaryDisplayNotFound,

  #[error("Not main thread")]
  NotMainThread,

  #[error("Display not found")]
  DisplayNotFound,

  #[error("Display device not found")]
  DisplayDeviceNotFound,

  #[error("Hardware enumeration failed")]
  HardwareEnumerationFailed,
}

pub type Result<T> = std::result::Result<T, Error>;
