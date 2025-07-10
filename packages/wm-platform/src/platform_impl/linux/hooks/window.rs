use tokio::sync::mpsc::error::SendError;

use super::Hook;
use crate::WindowEvent;

#[derive(Debug)]
pub struct WindowEventHook {
  rx: tokio::sync::mpsc::UnboundedReceiver<WindowEvent>,
}

#[derive(Debug)]
pub struct EventThreadWindowEventHook {
  tx: tokio::sync::mpsc::UnboundedSender<WindowEvent>,
}

impl Hook for EventThreadWindowEventHook {
  type Event = WindowEvent;

  fn dispatch(
    &self,
    event: Self::Event,
  ) -> Result<(), SendError<Self::Event>> {
    self.tx.send(event)
  }
}
