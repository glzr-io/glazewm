use super::Hook;

#[derive(Debug)]
pub struct DisplayHook {
  rx: tokio::sync::mpsc::UnboundedReceiver<crate::DisplayEvent>,
}

impl DisplayHook {
  #[must_use]
  pub fn new(
    rx: tokio::sync::mpsc::UnboundedReceiver<crate::DisplayEvent>,
  ) -> Self {
    Self { rx }
  }

  pub async fn next_event(&mut self) -> Option<crate::DisplayEvent> {
    self.rx.recv().await
  }
}

#[derive(Debug)]
pub struct EventThreadDisplayHook {
  tx: tokio::sync::mpsc::UnboundedSender<crate::DisplayEvent>,
}

impl EventThreadDisplayHook {
  #[must_use]
  pub fn new(
    tx: tokio::sync::mpsc::UnboundedSender<crate::DisplayEvent>,
  ) -> Self {
    Self { tx }
  }
}

impl Hook for EventThreadDisplayHook {
  type Event = crate::DisplayEvent;

  fn dispatch(
    &self,
    event: Self::Event,
  ) -> Result<(), tokio::sync::mpsc::error::SendError<Self::Event>> {
    self.tx.send(event)
  }
}
