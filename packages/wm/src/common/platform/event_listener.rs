use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{
  mpsc::{self, UnboundedReceiver},
  Mutex,
};

use crate::user_config::{KeybindingConfig, UserConfig};

use super::{EventWindow, NativeWindow};

#[derive(Debug)]
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
  pub event_rx: UnboundedReceiver<PlatformEvent>,
  event_window: EventWindow,
}

impl EventListener {
  /// Starts listening for platform events.
  pub async fn start(config: Arc<Mutex<UserConfig>>) -> Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let config = config.lock().await;

    let event_window = EventWindow::new(
      event_tx,
      config.keybindings.clone(),
      config.general.focus_follows_cursor,
    );

    Ok(Self {
      event_rx,
      event_window,
    })
  }

  pub fn update(&mut self, config: &UserConfig) {
    self.event_window.update(
      config.keybindings.clone(),
      config.general.focus_follows_cursor,
    );
  }
}
