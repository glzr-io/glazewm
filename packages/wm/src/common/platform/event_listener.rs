use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{
  mpsc::{self, UnboundedReceiver},
  Mutex,
};
use wineventhook::{EventFilter, WindowEvent, WindowEventHook};

use crate::user_config::{KeybindingConfig, UserConfig};

use super::{EventWindow, NativeWindow};

pub enum PlatformEvent {
  DisplaySettingsChanged,
  KeybindingTriggered(KeybindingConfig),
  MouseMove,
  WindowDestroyed(NativeWindow),
  WindowFocused(NativeWindow),
  WindowHidden(NativeWindow),
  WindowLocationChanged(NativeWindow),
  WindowMinimized(NativeWindow),
  WindowMinimizeEnded(NativeWindow),
  WindowMovedOrResized(NativeWindow),
  WindowShown(NativeWindow),
  WindowTitleChanged(NativeWindow),
}

pub struct EventListener {
  config: Arc<Mutex<UserConfig>>,
  config_changes_rx: UnboundedReceiver<UserConfig>,
  pub event_rx: UnboundedReceiver<WindowEvent>,
  event_window: EventWindow,
}

impl EventListener {
  /// Start listening for platform events.
  pub async fn start(
    config: Arc<Mutex<UserConfig>>,
    config_changes_rx: UnboundedReceiver<UserConfig>,
  ) -> Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let event_window = EventWindow::new();

    Ok(Self {
      config,
      config_changes_rx,
      event_rx,
      event_window,
    })
  }
}

impl Drop for EventListener {
  fn drop(&mut self) {
    // TODO
    // self.hook.unhook().unwrap();
  }
}
