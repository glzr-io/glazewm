use anyhow::bail;
use tokio::sync::mpsc;

use crate::platform_impl;

pub struct PlatformHookInstaller {
  installed_tx: mpsc::UnboundedSender<EventLoopDispatcher>,
}

impl PlatformHookInstaller {
  /// Install on main thread (MacOS only).
  #[cfg(target_os = "macos")]
  pub async fn install_on_main_thread(mut self) -> anyhow::Result<()> {
    // Verify we're on the main thread.
    if !platform_impl::is_main_thread() {
      bail!("Must be installed on the MacOS main thread.");
    }

    // TODO: Can probably in-line the implementation here.
    platform_impl::platform_thread_main(self.installed_tx).await;
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
