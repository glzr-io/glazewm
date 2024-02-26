use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{
  mpsc::{self, UnboundedReceiver},
  Mutex,
};
use wineventhook::{EventFilter, WindowEvent, WindowEventHook};

use crate::user_config::UserConfig;

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
  config: Arc<Mutex<UserConfig>>,
  config_changes_rx: UnboundedReceiver<UserConfig>,
  pub event_rx: UnboundedReceiver<WindowEvent>,
  hook: WindowEventHook,
}

impl EventListener {
  /// Start listening for platform events.
  pub async fn start(
    config: Arc<Mutex<UserConfig>>,
    config_changes_rx: UnboundedReceiver<UserConfig>,
  ) -> Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let hook =
      WindowEventHook::hook(EventFilter::default(), event_tx).await?;

    Ok(Self {
      config,
      config_changes_rx,
      event_rx,
      hook,
    })
  }
}

impl Drop for EventListener {
  fn drop(&mut self) {
    // TODO
    // self.hook.unhook().unwrap();
  }
}
