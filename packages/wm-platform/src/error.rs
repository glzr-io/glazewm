#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),

  #[error(transparent)]
  #[cfg(target_os = "windows")]
  Windows(#[from] windows::core::Error),

  #[error("Accessibility operation failed for attribute {0} with error code: {1}")]
  #[cfg(target_os = "macos")]
  Accessibility(String, i32),

  #[error("Invalid pointer: {0}")]
  InvalidPointer(String),

  #[error("AXValue creation failed: {0}")]
  #[cfg(target_os = "macos")]
  AXValueCreation(String),

  #[error(transparent)]
  ChannelRecv(#[from] std::sync::mpsc::RecvError),

  #[error("Channel send error: {0}")]
  ChannelSend(String),

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

  #[error("Window enumeration failed")]
  WindowEnumerationFailed,

  #[error("Thread error: {0}")]
  Thread(String),

  #[error("Window message error: {0}")]
  WindowMessage(String),

  #[error("Platform error: {0}")]
  Platform(String),

  #[error("Event loop has been stopped")]
  EventLoopStopped,

  #[error("Keybinding is empty")]
  InvalidKeybinding,
}

pub type Result<T> = std::result::Result<T, Error>;
