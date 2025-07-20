use anyhow::bail;
use objc2::{msg_send, ClassType};
use objc2_foundation::NSThread;
use tokio::sync::mpsc;

use crate::platform_impl;

pub struct PlatformHookInstaller {
  installed_tx: mpsc::UnboundedSender<EventLoopDispatcher>,
}

impl PlatformHookInstaller {
  pub fn new(
    installed_tx: mpsc::UnboundedSender<EventLoopDispatcher>,
  ) -> Self {
    Self { installed_tx }
  }

  /// Creates and runs a dedicated event loop on the current thread.
  /// This method blocks until the hook is shut down.
  ///
  /// To instead install the hook on an existing event loop, use
  /// [`PlatformHookInstaller::install_on_main_thread`] or
  /// [`PlatformHookInstaller::install_with_subclass`] instead.
  ///
  /// # Platform-specific behavior
  ///
  /// - **macOS**: Must be called from the main thread. Runs
  ///   `CFRunLoopRun()`. Note that macOS only allows a single main run
  ///   loop within a process.
  /// - **Windows**: Can be called from any thread. Runs a Win32 message
  ///   loop.
  pub fn run_dedicated_loop(self) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    self.run_macos_main_loop();

    #[cfg(target_os = "windows")]
    self.run_windows_message_loop();

    Ok(())
  }

  /// Install on main thread (MacOS only).
  #[cfg(target_os = "macos")]
  pub async fn install_on_main_thread(mut self) -> anyhow::Result<()> {
    let is_main_thread: bool =
      unsafe { msg_send![NSThread::class(), isMainThread] };

    // Verify we're on the main thread.
    if !is_main_thread {
      bail!("Must be installed on the MacOS main thread.");
    }

    // TODO: Can probably in-line the implementation here.
    // platform_impl::platform_thread_main(self.installed_tx).await;
    Ok(())
  }

  /// Install with window subclassing (Windows only).
  #[cfg(windows)]
  pub async fn install_with_subclass(
    mut self,
    hwnd: WindowHandle,
  ) -> anyhow::Result<()> {
    // TODO: Can probably in-line the implementation here. Alternatively,
    // create static methods on `EventLoopDispatcher` to install the hook.
    platform_impl::setup_window_subclass(hwnd, self.installed_tx)?;
    Ok(())
  }
}
