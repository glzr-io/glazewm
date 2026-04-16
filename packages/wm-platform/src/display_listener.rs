use tokio::sync::mpsc;

use crate::{platform_impl, Dispatcher};

/// The kind of change that triggered a display settings event.
#[derive(Clone, Debug)]
pub enum DisplayEventKind {
  /// A display was connected/disconnected, or its resolution or arrangement
  /// changed (`WM_DISPLAYCHANGE` on Windows).
  DisplayChanged,
  /// The working area changed, e.g. due to a taskbar resize or appbar
  /// registration (`WM_SETTINGCHANGE` with `SPI_SETWORKAREA` on Windows).
  WorkAreaChanged,
  /// A non-display device node changed (`WM_DEVICECHANGE` on Windows).
  /// This fires for any device, including USB peripherals and audio devices.
  DeviceChanged,
}

/// A listener for system-wide display setting changes.
///
/// Detects changes to display configuration including resolution changes,
/// display connections/disconnections, and working area changes.
pub struct DisplayListener {
  event_rx: mpsc::UnboundedReceiver<DisplayEventKind>,

  /// Inner platform-specific display listener.
  inner: platform_impl::DisplayListener,
}

impl DisplayListener {
  /// Creates a new [`DisplayListener`].
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let inner = platform_impl::DisplayListener::new(event_tx, dispatcher)?;
    Ok(Self { event_rx, inner })
  }

  /// Returns when the next display settings change is detected.
  ///
  /// Returns `None` if the channel has been closed.
  pub async fn next_event(&mut self) -> Option<DisplayEventKind> {
    self.event_rx.recv().await
  }

  /// Terminates the display listener.
  pub fn terminate(&mut self) -> crate::Result<()> {
    self.inner.terminate()
  }
}
