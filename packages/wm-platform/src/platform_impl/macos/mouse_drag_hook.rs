use tokio::sync::mpsc;

use crate::{Dispatcher, DragModifier, MouseDragEvent, WindowId};

/// Inert macOS stand-in for the Windows mouse drag hook.
///
/// Grab-and-move relies on `WH_MOUSE_LL` (a Windows-only mechanism), so
/// this stub accepts all calls, keeps the event sender alive (so the
/// receiver never yields `None` and busy-loops a `select!` arm), and
/// never sends an event.
#[derive(Debug)]
pub struct MouseDragHook {
  _event_tx: mpsc::UnboundedSender<MouseDragEvent>,
}

impl MouseDragHook {
  /// Implements [`crate::MouseDragHook::new`].
  #[allow(clippy::unnecessary_wraps)]
  pub fn new(
    _modifier: &DragModifier,
    _enabled: bool,
    event_tx: mpsc::UnboundedSender<MouseDragEvent>,
    _dispatcher: &Dispatcher,
  ) -> crate::Result<Self> {
    Ok(Self {
      _event_tx: event_tx,
    })
  }

  /// Implements [`crate::MouseDragHook::set_enabled`].
  pub fn set_enabled(&self, _enabled: bool) {}

  /// Implements [`crate::MouseDragHook::set_modifier`].
  pub fn set_modifier(&self, _modifier: &DragModifier) {}

  /// Implements [`crate::MouseDragHook::update_managed_windows`].
  pub fn update_managed_windows(&self, _handles: &[WindowId]) {}

  /// Implements [`crate::MouseDragHook::terminate`].
  #[allow(clippy::unnecessary_wraps)]
  pub fn terminate(&mut self) -> crate::Result<()> {
    Ok(())
  }
}
