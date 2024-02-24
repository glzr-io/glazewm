use anyhow::Result;
use tokio::sync::mpsc::{self, UnboundedReceiver};
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
  pub event_rx: UnboundedReceiver<WindowEvent>,
  hook: WindowEventHook,
}

impl EventListener {
  /// Start listening for platform events.
  pub async fn start() -> Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let hook =
      WindowEventHook::hook(EventFilter::default(), event_tx).await?;

    Ok(Self { event_rx, hook })
  }
}

impl Drop for EventListener {
  fn drop(&mut self) {
    // TODO
    // self.hook.unhook().unwrap();
  }
}
