use tokio::sync::mpsc::{self, Receiver, Sender};
use wineventhook::{EventFilter, WindowEvent, WindowEventHook};

pub enum PlatformEvent {
  DisplaySettingsChanged,
  MouseMove,
  WindowDestroyed,
  WindowFocused,
  WindowHidden,
  WindowLocationChanged,
  WindowMinimizeEnded,
  WindowMinimized,
  WindowMovedOrResized,
  WindowShown,
  WindowTitleChanged,
}

pub struct EventListener {
  event_tx: Sender<WindowEvent>,
  pub event_rx: Receiver<WindowEvent>,
  hook: WindowEventHook,
}

impl EventListener {
  /// Start listening for platform events.
  pub async fn start() -> Self {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    let hook =
      WindowEventHook::hook(EventFilter::default(), event_tx).await?;

    Self {
      event_tx,
      event_rx,
      hook,
    }
  }
}

impl Drop for EventListener {
  fn drop(&mut self) {
    self.hook.unhook().unwrap();
  }
}
