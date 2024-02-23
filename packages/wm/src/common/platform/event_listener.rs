use anyhow::Result;

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
  pub fn new() -> Self {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    Self { event_tx, event_rx }
  }

  pub async fn start(&mut self) -> Self {
    self.hook =
      WindowEventHook::hook(EventFilter::default(), self.event_tx).await?;

    self
  }
}

impl Drop for EventListener {
  fn drop(&mut self) {
    self.hook.unhook().unwrap();
  }
}
