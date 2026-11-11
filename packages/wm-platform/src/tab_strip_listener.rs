use tokio::sync::mpsc;

use crate::{platform_impl, Dispatcher, TabStripEvent};

/// A listener for events emitted by [`TabStrip`](crate::TabStrip)s.
pub struct TabStripListener {
  event_rx: mpsc::UnboundedReceiver<TabStripEvent>,

  /// Inner platform-specific tab strip listener.
  #[allow(dead_code)]
  inner: platform_impl::TabStripListener,
}

impl TabStripListener {
  /// Creates a new tab strip listener.
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let inner =
      platform_impl::TabStripListener::new(event_tx, dispatcher)?;

    Ok(Self { event_rx, inner })
  }

  /// Returns the next event from any tab strip.
  ///
  /// Returns `None` if the channel has been closed.
  pub async fn next_event(&mut self) -> Option<TabStripEvent> {
    self.event_rx.recv().await
  }
}
