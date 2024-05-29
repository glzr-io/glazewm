use std::sync::Arc;

use anyhow::Result;
use tokio::sync::{
  mpsc::{self, UnboundedReceiver},
  Mutex,
};

use crate::{
  common::Point,
  user_config::{BindingModeConfig, KeybindingConfig, UserConfig},
};

use super::{EventWindow, EventWindowOptions, NativeWindow};

#[derive(Debug)]
pub enum PlatformEvent {
  PowerModeChanged,
  DisplaySettingsChanged,
  KeybindingTriggered(KeybindingConfig),
  MouseMove(MouseMoveEvent),
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

#[derive(Debug, Clone)]
pub struct MouseMoveEvent {
  /// Location of mouse with 0,0 being the top-left corner of the primary
  /// monitor.
  pub point: Point,

  /// Whether either left or right-click is currently pressed.
  pub is_mouse_down: bool,
}

pub struct EventListener {
  pub event_rx: UnboundedReceiver<PlatformEvent>,
  event_window: EventWindow,
}

impl EventListener {
  /// Initializes listener for platform events.
  ///
  /// Returns an instance of `EventListener`.
  pub fn start(config: &UserConfig) -> Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let event_window = EventWindow::new(
      event_tx,
      EventWindowOptions {
        keybindings: config.value.keybindings.clone(),
        enable_mouse_events: config.value.general.focus_follows_cursor,
      },
    );

    Ok(Self {
      event_rx,
      event_window,
    })
  }

  /// Updates the event listener with the latest user config and the
  /// currently active binding modes.
  pub fn update(
    &mut self,
    config: &UserConfig,
    binding_modes: &Vec<BindingModeConfig>,
  ) {
    // TODO: Modify keybindings based on active binding modes.
    self
      .event_window
      .update_keybindings(config.value.keybindings.clone());

    self
      .event_window
      .enable_mouse_listener(config.value.general.focus_follows_cursor);
  }
}
