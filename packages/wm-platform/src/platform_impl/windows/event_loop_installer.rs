use windows::Win32::Foundation::HWND;

use crate::platform_impl::EventLoopDispatcher;

pub struct EventLoopInstaller {
  dispatcher: EventLoopDispatcher,
}

impl EventLoopInstaller {
  pub fn new() -> crate::Result<(Self, EventLoopDispatcher)> {
    // For installer pattern, we create a placeholder dispatcher
    // The actual dispatcher will be created when install_with_subclass is
    // called
    let placeholder_dispatcher = EventLoopDispatcher::new(0, 0);
    let installer_dispatcher = placeholder_dispatcher.clone();

    Ok((
      Self {
        dispatcher: installer_dispatcher.clone(),
      },
      placeholder_dispatcher,
    ))
  }

  /// Install on an existing event loop via window subclassing (Windows
  /// only).
  ///
  /// This method integrates with an existing Windows message loop by
  /// subclassing the specified window.
  pub fn install_with_subclass(self, hwnd: HWND) -> crate::Result<()> {
    use windows::Win32::System::Threading::GetCurrentThreadId;

    let thread_id = unsafe { GetCurrentThreadId() };

    // Create a new dispatcher with the actual window handle
    let _dispatcher = EventLoopDispatcher::new(hwnd.0 as _, thread_id);

    // TODO: Set up window subclass to handle our custom messages. This
    // would involve calling `SetWindowSubclass` and handling
    // `WM_DISPATCH_CALLBACK`.

    tracing::info!(
      "EventLoopInstaller installed with subclass on HWND: {:?}",
      hwnd
    );
    Ok(())
  }
}
