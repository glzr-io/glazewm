use objc2::{msg_send, ClassType};
use objc2_foundation::NSThread;
use tokio::sync::oneshot;

use crate::platform_impl::{EventLoop, EventLoopDispatcher};

/// Installer for setting up platform-specific event loop integration.
///
/// This struct is used to install a platform hook on either the current
/// thread with a dedicated event loop, or an existing event loop.
pub struct PlatformHookInstaller {
  dispatcher_tx: oneshot::Sender<EventLoopDispatcher>,
}

impl PlatformHookInstaller {
  pub(crate) fn new(
    dispatcher_tx: oneshot::Sender<EventLoopDispatcher>,
  ) -> Self {
    Self { dispatcher_tx }
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
    // Create event loop on current thread.
    let (event_loop, dispatcher) = EventLoop::new()?;

    // Send dispatcher back to `PlatformHook`.
    if self.dispatcher_tx.send(dispatcher).is_err() {
      anyhow::bail!("Failed to send dispatcher back to PlatformHook");
    }

    // Run the event loop (blocks until shutdown).
    event_loop.run();
    Ok(())
  }

  /// Install on main thread (MacOS only).
  #[cfg(target_os = "macos")]
  pub fn install_on_main_thread(self) -> anyhow::Result<()> {
    let is_main_thread: bool =
      unsafe { msg_send![NSThread::class(), isMainThread] };

    // Verify we're on the main thread.
    if !is_main_thread {
      anyhow::bail!("Must be installed on the MacOS main thread.");
    }

    // TODO: Can probably in-line the implementation here.
    // platform_impl::platform_thread_main(self.installed_tx).await;
    Ok(())
  }

  /// Install on an existing event loop via window subclassing (Windows
  /// only).
  #[cfg(windows)]
  pub fn install_with_subclass(
    mut self,
    hwnd: WindowHandle,
  ) -> anyhow::Result<()> {
    // Create a dispatcher that works with the existing window.
    let dispatcher = EventLoopDispatcher {
      message_window_handle: hwnd,
      thread_id: unsafe { GetCurrentThreadId() },
      shutdown_flag: Arc::new(AtomicBool::new(false)),
    };

    // Send dispatcher back to `PlatformHook`.
    if self.dispatcher_tx.send(dispatcher).is_err() {
      anyhow::bail!("Failed to send dispatcher back to PlatformHook");
    }

    // TODO: Set up window subclass to handle our custom messages. This
    // would involve calling `SetWindowSubclass` and handling
    // `WM_DISPATCH_CALLBACK`.

    Ok(())
  }
}
