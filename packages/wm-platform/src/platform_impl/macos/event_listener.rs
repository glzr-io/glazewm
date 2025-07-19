use tokio::sync::mpsc::{self, UnboundedReceiver};
use wm_common::{
  BindingModeConfig, InvokeCommand, KeybindingConfig, ParsedConfig, Point,
};

use super::NativeWindow;

pub enum PlatformEvent {
  DisplaySettingsChanged,
  WindowDestroyed(NativeWindow),
  WindowFocused(NativeWindow),
  WindowHidden(NativeWindow),
  WindowLocationChanged(NativeWindow),
  WindowMinimized(NativeWindow),
  WindowMinimizeEnded(NativeWindow),
  WindowMovedOrResizedEnd(NativeWindow),
  WindowMovedOrResizedStart(NativeWindow),
  WindowShown(NativeWindow),
  WindowTitleChanged(NativeWindow),
}

pub struct EventListener {
  pub event_rx: UnboundedReceiver<PlatformEvent>,
}

impl EventListener {
  /// Initializes listener for platform events.
  ///
  /// Returns an instance of `EventListener`.
  pub fn start(config: &ParsedConfig) -> anyhow::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    // TODO

    Ok(Self { event_rx })
  }
}
