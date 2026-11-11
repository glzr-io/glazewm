//! macOS implementation of [`TabStrip`](crate::TabStrip).
//!
//! Not yet implemented. Strips are never shown, so groups function
//! without a visible header on macOS.

use tokio::sync::mpsc;

use crate::{Dispatcher, TabStripEvent, TabStripId, TabStripState};

/// Platform-specific implementation of [`TabStrip`](crate::TabStrip).
pub(crate) struct TabStrip;

impl TabStrip {
  /// Implements [`TabStrip::new`](crate::TabStrip::new).
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn new(_dispatcher: &Dispatcher) -> crate::Result<Self> {
    Ok(Self)
  }

  /// Implements [`TabStrip::id`](crate::TabStrip::id).
  pub(crate) fn id(&self) -> TabStripId {
    TabStripId(0)
  }

  /// Implements [`TabStrip::set_state`](crate::TabStrip::set_state).
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn set_state(
    &mut self,
    _state: &TabStripState,
  ) -> crate::Result<()> {
    Ok(())
  }
}

/// Platform-specific implementation of
/// [`TabStripListener`](crate::TabStripListener).
pub(crate) struct TabStripListener;

impl TabStripListener {
  /// Implements [`TabStripListener::new`](crate::TabStripListener::new).
  #[allow(clippy::unnecessary_wraps)]
  pub(crate) fn new(
    _event_tx: mpsc::UnboundedSender<TabStripEvent>,
    _dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    Ok(Self)
  }
}
