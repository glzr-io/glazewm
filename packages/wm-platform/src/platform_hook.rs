use std::time::Duration;

use anyhow::{bail, Context};
use tokio::{sync::mpsc, task};
use tracing::warn;
use wm_common::{
  Color, CornerStyle, Delta, HideMethod, LengthValue, Memo, OpacityValue,
  Rect, RectDelta, WindowState,
};

#[derive(Clone, Debug)]
pub struct PlatformHook {
  event_loop: Option<EventLoop>,
  dispatcher: Option<EventLoopDispatcher>,
}

impl PlatformHook {
  /// Creates a new `PlatformHook` instance with its own event loop running
  /// on a separate, dedicated thread.
  ///
  /// ## Platform-specific
  ///
  /// - Windows: Creates a new Win32 message loop.
  /// - MacOS: Creates a new *main* run loop (`CFRunLoopGetMain()`). Note
  ///   that MacOS only allows a single main run loop in a process, so if
  ///   you'd like to run your own main run loop, set up the hook using
  ///   [`PlatformHook::remote`] instead.
  #[must_use]
  pub fn dedicated() -> Self {
    let (event_loop, dispatcher) = EventLoop::new();

    Self {
      event_loop,
      dispatcher,
    }
  }

  /// Creates a new `PlatformHook` instance to be installed on a target
  /// thread. The `PlatformHookInstaller` returned is used to install
  /// the hook on the target thread.
  #[must_use]
  pub fn remote() -> (Self, PlatformHookInstaller) {
    let (installed_tx, installed_rx) = mpsc::unbounded_channel();
    let installer = PlatformHookInstaller::new(installed_tx);

    (
      Self {
        event_loop: None,
        dispatcher: None,
      },
      installer,
    )
  }

  /// Creates a new [`MouseListener`] instance.
  ///
  /// This method will wait for the platform hook to be installed.
  pub async fn create_mouse_listener(
    &mut self,
  ) -> anyhow::Result<MouseListener> {
    let dispatcher = self.resolve_dispatcher().await?;
    MouseListener::new(dispatcher)
  }

  async fn resolve_dispatcher(
    &mut self,
  ) -> anyhow::Result<EventLoopDispatcher> {
    // TODO: Wait for the dispatcher to be installed.
    todo!()
  }
}

impl Drop for PlatformHook {
  fn drop(&mut self) {
    if let Some(event_loop) = self.event_loop.take() {
      event_loop.stop();
    }
  }
}
