use tokio::sync::mpsc::error::SendError;

use super::Hook;
use crate::MouseEvent;

#[derive(Debug)]
pub struct MouseHook {
  rx: tokio::sync::mpsc::UnboundedReceiver<MouseEvent>,
}

impl MouseHook {
  #[must_use]
  pub fn new(
    rx: tokio::sync::mpsc::UnboundedReceiver<MouseEvent>,
  ) -> Self {
    Self { rx }
  }

  pub async fn next_event(&mut self) -> Option<MouseEvent> {
    self.rx.recv().await
  }
}

#[derive(Debug)]
pub struct EventThreadMouseHook {
  tx: tokio::sync::mpsc::UnboundedSender<MouseEvent>,
  enable: bool,
}

impl EventThreadMouseHook {
  #[must_use]
  pub fn new(tx: tokio::sync::mpsc::UnboundedSender<MouseEvent>) -> Self {
    Self { tx, enable: false }
  }

  pub fn update(&mut self, enable: bool) {
    self.enable = enable;
  }
}

impl Hook for EventThreadMouseHook {
  type Event = MouseEvent;

  fn dispatch(
    &self,
    event: Self::Event,
  ) -> Result<(), SendError<Self::Event>> {
    if self.enable {
      self.tx.send(event)
    } else {
      Ok(())
    }
  }
}
