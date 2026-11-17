use tokio::sync::mpsc;

#[cfg(target_os = "windows")]
use crate::platform_impl;
use crate::{AccentPalette, Dispatcher};

/// A listener for changes to the system accent palette.
///
/// Only emits when the palette actually differs from the previously seen
/// one.
///
/// # Platform-specific
///
/// - **macOS**: No events are emitted.
pub struct AccentPaletteListener {
  event_rx: mpsc::UnboundedReceiver<AccentPalette>,

  /// Inner platform-specific accent palette listener.
  #[cfg(target_os = "windows")]
  inner: platform_impl::AccentPaletteListener,

  /// Kept alive such that `next_event` stays pending.
  #[cfg(target_os = "macos")]
  _event_tx: mpsc::UnboundedSender<AccentPalette>,
}

impl AccentPaletteListener {
  /// Creates a new [`AccentPaletteListener`].
  // LINT: `dispatcher` is only used on Windows.
  #[cfg_attr(not(target_os = "windows"), allow(unused_variables))]
  pub fn new(dispatcher: &Dispatcher) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    #[cfg(target_os = "windows")]
    let inner =
      platform_impl::AccentPaletteListener::new(event_tx, dispatcher)?;

    Ok(Self {
      event_rx,
      #[cfg(target_os = "windows")]
      inner,
      #[cfg(target_os = "macos")]
      _event_tx: event_tx,
    })
  }

  /// Returns when the next accent palette change is detected.
  ///
  /// Returns `None` if the channel has been closed.
  pub async fn next_event(&mut self) -> Option<AccentPalette> {
    self.event_rx.recv().await
  }

  /// Terminates the accent palette listener.
  pub fn terminate(&mut self) -> crate::Result<()> {
    #[cfg(target_os = "windows")]
    self.inner.terminate()?;

    Ok(())
  }
}
