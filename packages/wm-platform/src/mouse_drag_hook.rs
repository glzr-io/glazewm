use tokio::sync::mpsc;

use crate::{platform_impl, Dispatcher, WindowId};

/// Modifier key that activates a grab-and-move drag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DragModifier {
  Alt,
  Win,
}

/// Lifecycle events for a hook-driven window drag.
#[derive(Clone, Copy, Debug)]
pub enum MouseDragEvent {
  /// A drag session started on the given window.
  Started { window_id: WindowId },

  /// The drag session on the given window ended (button released, or
  /// the hook was disabled mid-drag).
  Ended { window_id: WindowId },
}

/// A hook for grab-and-move: holding a modifier key and left-clicking
/// anywhere on a managed window starts a drag that moves the window
/// with the cursor until the button is released.
///
/// The WM keeps the hook's set of managed window handles up to date;
/// unmanaged/ignored windows pass clicks through untouched, so window
/// rules (e.g. `ignore` for games) are respected automatically. Drag
/// lifecycle events are consumed via [`MouseDragHook::next_event`] so
/// the WM can apply the same interactive-drag handling as a native
/// title-bar drag.
///
/// # Platform-specific
///
/// Windows-only. On macOS this is an inert no-op that never yields
/// events.
#[derive(Debug)]
pub struct MouseDragHook {
  /// Receiver for outgoing drag lifecycle events.
  event_rx: mpsc::UnboundedReceiver<MouseDragEvent>,

  /// Inner platform-specific hook.
  inner: platform_impl::MouseDragHook,
}

impl MouseDragHook {
  /// Creates a new [`MouseDragHook`].
  pub fn new(
    modifier: &DragModifier,
    enabled: bool,
    dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let inner = platform_impl::MouseDragHook::new(
      modifier, enabled, event_tx, dispatcher,
    )?;

    Ok(Self { event_rx, inner })
  }

  /// Returns the next drag lifecycle event from the hook.
  ///
  /// This will block until an event is available.
  pub async fn next_event(&mut self) -> Option<MouseDragEvent> {
    self.event_rx.recv().await
  }

  /// Enables or disables drag interception.
  pub fn set_enabled(&self, enabled: bool) {
    self.inner.set_enabled(enabled);
  }

  /// Updates the modifier key that activates a drag.
  pub fn set_modifier(&self, modifier: &DragModifier) {
    self.inner.set_modifier(modifier);
  }

  /// Replaces the set of managed window IDs.
  pub fn update_managed_windows(&self, handles: &[WindowId]) {
    self.inner.update_managed_windows(handles);
  }

  /// Terminates the hook.
  pub fn terminate(&mut self) -> crate::Result<()> {
    self.inner.terminate()
  }
}
