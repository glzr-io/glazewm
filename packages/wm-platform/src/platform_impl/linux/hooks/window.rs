use tokio::sync::mpsc::error::SendError;

use super::Hook;
use crate::{WindowEvent, WindowEventType};

#[derive(Debug)]
pub struct WindowEventHook {
  rx: tokio::sync::mpsc::UnboundedReceiver<WindowEvent>,
}

impl WindowEventHook {
  #[must_use]
  pub fn new(
    rx: tokio::sync::mpsc::UnboundedReceiver<WindowEvent>,
  ) -> Self {
    Self { rx }
  }

  pub async fn next_event(&mut self) -> Option<WindowEvent> {
    self.rx.recv().await
  }
}

#[derive(Debug)]
pub struct EventThreadWindowEventHook {
  tx: tokio::sync::mpsc::UnboundedSender<WindowEvent>,
  events: WindowEventType,
}

impl EventThreadWindowEventHook {
  #[must_use]
  pub fn new(
    tx: tokio::sync::mpsc::UnboundedSender<WindowEvent>,
    events: WindowEventType,
  ) -> Self {
    Self { tx, events }
  }
}

impl Hook for EventThreadWindowEventHook {
  type Event = WindowEvent;

  fn dispatch(
    &self,
    event: Self::Event,
  ) -> Result<(), SendError<Self::Event>> {
    if self.events.intersects(event.get_type()) {
      self.tx.send(event)
    } else {
      Ok(())
    }
  }
}
