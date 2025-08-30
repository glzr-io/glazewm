#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;

use crate::{platform_impl, Dispatcher};

/// An installer for integrating with existing event loops.
///
/// This allows the platform integration to work with existing event loops
/// rather than requiring a dedicated event loop. The installer provides
/// platform-specific installation methods.
pub struct EventLoopInstaller {
  inner: platform_impl::EventLoopInstaller,
}

impl EventLoopInstaller {
  /// Creates a new installer and dispatcher for integrating with existing
  /// event loops.
  ///
  /// The dispatcher can be used immediately to create listeners and query
  /// system state, but events will only be received after calling the
  /// appropriate install method.
  pub fn new() -> crate::Result<(Self, Dispatcher)> {
    let (inner_installer, inner_dispatcher) =
      platform_impl::EventLoopInstaller::new()?;
    let dispatcher = Dispatcher::new(inner_dispatcher);

    Ok((
      Self {
        inner: inner_installer,
      },
      dispatcher,
    ))
  }

  /// Install on the main thread (macOS only).
  ///
  /// This method integrates with the existing CFRunLoop on the main
  /// thread. It must be called from the main thread.
  ///
  /// # Platform-specific
  ///
  /// - **macOS**: Must be called from the main thread.
  #[cfg(target_os = "macos")]
  pub fn install(self) -> crate::Result<()> {
    self.inner.install()
  }

  /// Install on an existing event loop via window subclassing (Windows
  /// only).
  ///
  /// This method integrates with an existing Windows message loop by
  /// subclassing the specified window.
  ///
  /// # Platform-specific
  ///
  /// - **Windows**: Integrates with existing message loop via subclassing.
  #[cfg(target_os = "windows")]
  pub fn install_with_subclass(self, hwnd: HWND) -> crate::Result<()> {
    self.inner.install_with_subclass(hwnd)
  }
}
