use wm_common::KeybindingConfig;

use super::Hook;
use crate::KeyboardEvent;

#[derive(Debug)]
pub struct KeyboardHook {
  rx: tokio::sync::mpsc::UnboundedReceiver<KeyboardEvent>,
}

impl KeyboardHook {
  #[must_use]
  pub fn new(
    rx: tokio::sync::mpsc::UnboundedReceiver<KeyboardEvent>,
  ) -> Self {
    Self { rx }
  }

  pub async fn next_event(&mut self) -> Option<KeyboardEvent> {
    self.rx.recv().await
  }
}

#[derive(Debug)]
pub struct EventThreadKeyboardHook {
  tx: tokio::sync::mpsc::UnboundedSender<KeyboardEvent>,
  keybinds: Vec<KeybindingConfig>,
}

impl EventThreadKeyboardHook {
  #[must_use]
  pub fn new(
    tx: tokio::sync::mpsc::UnboundedSender<KeyboardEvent>,
    keybinds: Vec<KeybindingConfig>,
  ) -> Self {
    Self { tx, keybinds }
  }

  pub fn update_keybinds(&mut self, keybinds: Vec<KeybindingConfig>) {
    self.keybinds = keybinds;
  }
}

impl Hook for EventThreadKeyboardHook {
  type Event = KeyboardEvent;

  fn dispatch(
    &self,
    event: Self::Event,
  ) -> Result<(), tokio::sync::mpsc::error::SendError<Self::Event>> {
    self.tx.send(event)
  }
}
