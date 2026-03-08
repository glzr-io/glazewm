use tokio::sync::mpsc;

use crate::{platform_event::MouseEvent, platform_impl, Dispatcher};

/// Available mouse events that [`MouseListener`] can listen for.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MouseEventKind {
  Move,
  LeftButtonDown,
  LeftButtonUp,
  RightButtonDown,
  RightButtonUp,
}

/// A listener for system-wide mouse events.
pub struct MouseListener {
  /// Receiver for outgoing mouse events.
  event_rx: mpsc::UnboundedReceiver<MouseEvent>,

  /// Inner platform-specific mouse listener.
  inner: platform_impl::MouseListener,
}

impl MouseListener {
  /// Creates a new [`MouseListener`] with the specified enabled events.
  pub fn new(
    enabled_events: &[MouseEventKind],
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let inner = platform_impl::MouseListener::new(
      enabled_events,
      event_tx,
      dispatcher,
    )?;

    Ok(Self { event_rx, inner })
  }

  /// Returns the next mouse event from the listener.
  ///
  /// This will block until a mouse event is available.
  pub async fn next_event(&mut self) -> Option<MouseEvent> {
    self.event_rx.recv().await
  }

  /// Enables or disables the underlying mouse listener.
  pub fn enable(&mut self, enabled: bool) -> crate::Result<()> {
    self.inner.enable(enabled)
  }

  /// Updates the set of enabled mouse events to listen for.
  pub fn set_enabled_events(
    &mut self,
    enabled_events: &[MouseEventKind],
  ) -> crate::Result<()> {
    self.inner.set_enabled_events(enabled_events)
  }

  /// Terminates the mouse listener.
  pub fn terminate(&mut self) -> crate::Result<()> {
    self.inner.terminate()
  }
}
