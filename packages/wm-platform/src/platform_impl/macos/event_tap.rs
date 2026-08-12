use objc2_core_foundation::{CFMachPort, CFRetained};
use objc2_core_graphics::{CGEvent, CGEventType};

use crate::{Dispatcher, ThreadBound};

/// Handle to an event tap that can be accessed from its callback.
#[derive(Debug, Default)]
pub(super) struct EventTapHandle {
  tap_port: Option<ThreadBound<CFRetained<CFMachPort>>>,
}

impl EventTapHandle {
  /// Stores a retained reference to the event tap on its owning thread.
  pub(super) fn set(
    &mut self,
    tap_port: &CFRetained<CFMachPort>,
    dispatcher: &Dispatcher,
  ) {
    self.tap_port =
      Some(ThreadBound::new(tap_port.clone(), dispatcher.clone()));
  }

  /// Re-enables the event tap when macOS reports that it was disabled.
  ///
  /// Returns whether the event is a tap-disabled notification and should
  /// therefore bypass normal event processing.
  pub(super) fn reenable_if_disabled(
    &self,
    event_type: CGEventType,
  ) -> bool {
    let reason = match event_type {
      CGEventType::TapDisabledByTimeout => "callback timeout",
      CGEventType::TapDisabledByUserInput => "user input",
      _ => return false,
    };

    match self.tap_port.as_ref().map(ThreadBound::get_ref) {
      Some(Ok(tap_port)) => {
        CGEvent::tap_enable(tap_port, true);
        tracing::warn!(reason, "Re-enabled disabled macOS event tap.");
      }
      Some(Err(err)) => {
        tracing::error!(
          reason,
          %err,
          "Failed to access disabled macOS event tap."
        );
      }
      None => {
        tracing::error!(
          reason,
          "Disabled macOS event tap is missing its callback handle."
        );
      }
    }

    true
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn identifies_event_tap_disabled_notifications() {
    let handle = EventTapHandle::default();

    assert!(handle.reenable_if_disabled(CGEventType::TapDisabledByTimeout));
    assert!(
      handle.reenable_if_disabled(CGEventType::TapDisabledByUserInput)
    );
  }

  #[test]
  fn ignores_normal_input_events() {
    let handle = EventTapHandle::default();

    assert!(!handle.reenable_if_disabled(CGEventType::KeyDown));
    assert!(!handle.reenable_if_disabled(CGEventType::MouseMoved));
  }
}
