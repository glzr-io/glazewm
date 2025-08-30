use crate::{platform_impl, Dispatcher};

/// An installer for integrating [`Dispatcher`] with an existing
/// event loop.
pub struct EventLoopInstaller;

impl EventLoopInstaller {
  /// Creates a new installer and dispatcher for integrating with an
  /// existing event loop.
  pub fn new() -> crate::Result<(Self, Dispatcher)> {
    let dispatcher = Dispatcher::new(None);
    Ok((Self, dispatcher))
  }

  /// Install on an existing event loop running on the main thread (macOS
  /// only).
  ///
  /// This method integrates with the existing `CFRunLoop` on the main
  /// thread.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on macOS.
  #[cfg(target_os = "macos")]
  pub fn install(self) -> crate::Result<()> {
    let _source = platform_impl::EventLoop::add_dispatch_source()?;

    // TODO: Need to send the source to the dispatcher.
    todo!();
  }

  /// Install on an existing event loop via window subclassing (Windows
  /// only).
  ///
  /// This method integrates with an existing Windows message loop by
  /// subclassing the specified window.
  ///
  /// # Platform-specific
  ///
  /// This method is only available on Windows.
  #[cfg(target_os = "windows")]
  pub fn install_with_subclass(self, hwnd: HWND) -> crate::Result<()> {
    self.inner.install_with_subclass(hwnd)
  }
}
