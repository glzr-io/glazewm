use std::time::Duration;

use tokio::sync::mpsc;

use crate::{platform_impl, Dispatcher};

/// Quiet period used to coalesce a burst of display setting changes into a
/// single event.
///
/// Connecting monitors or resuming from sleep emits a rapid burst of OS
/// display messages while the topology settles. Reacting to each one causes
/// workspaces and windows to be repeatedly reshuffled.
const DEBOUNCE_DURATION: Duration = Duration::from_millis(250);

/// A listener for system-wide display setting changes.
///
/// Detects changes to display configuration including resolution changes,
/// display connections/disconnections, and working area changes.
///
/// Bursts of changes are debounced: a single event is surfaced only once no
/// further change has been seen for `DEBOUNCE_DURATION`.
pub struct DisplayListener {
  event_rx: mpsc::UnboundedReceiver<()>,

  /// Inner platform-specific display listener.
  inner: platform_impl::DisplayListener,
}

impl DisplayListener {
  /// Creates a new [`DisplayListener`].
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    // Raw events straight from the platform listener.
    let (raw_tx, raw_rx) = mpsc::unbounded_channel();
    // Coalesced events surfaced to consumers.
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let inner = platform_impl::DisplayListener::new(raw_tx, dispatcher)?;

    // Debounce in a dedicated task that owns the raw receiver. This must
    // not run inside the main event loop's `select!`, since that would
    // cancel the in-progress debounce (dropping already-received events)
    // whenever another branch fires mid-burst. The task ends on its own
    // once `inner` is dropped and the raw channel closes.
    tokio::spawn(debounce_loop(raw_rx, event_tx));

    Ok(Self { event_rx, inner })
  }

  /// Returns when the next (debounced) display settings change is detected.
  ///
  /// Returns `None` if the channel has been closed.
  pub async fn next_event(&mut self) -> Option<()> {
    self.event_rx.recv().await
  }

  /// Terminates the display listener.
  pub fn terminate(&mut self) -> crate::Result<()> {
    self.inner.terminate()
  }
}

/// Coalesces bursts of raw display change events, emitting a single event on
/// `event_tx` once `DEBOUNCE_DURATION` elapses without a further change.
///
/// Returns when either channel closes.
async fn debounce_loop(
  mut raw_rx: mpsc::UnboundedReceiver<()>,
  event_tx: mpsc::UnboundedSender<()>,
) {
  // Block until the first event of a burst arrives.
  while raw_rx.recv().await.is_some() {
    // Keep draining until there is a quiet gap, coalescing the burst.
    loop {
      match tokio::time::timeout(DEBOUNCE_DURATION, raw_rx.recv()).await {
        // Another change arrived within the window; keep waiting.
        Ok(Some(())) => continue,
        // Quiet gap reached (`Err`) or channel closed (`Ok(None)`).
        Ok(None) | Err(_) => break,
      }
    }

    // Surface a single coalesced event; stop if no consumer remains.
    if event_tx.send(()).is_err() {
      break;
    }
  }
}
