/// How to handle a native focus event for a window that still has native
/// focus.
#[derive(Debug, PartialEq, Eq)]
pub enum FocusAction {
  /// Drop the event without changing WM state.
  Ignore,

  /// Reassign focus to the WM's focused container.
  RestorePrevious,

  /// Sync the WM's focus to the natively focused window.
  Accept,
}

/// Decides how to handle a native focus event.
///
/// An explicit application activation (e.g. Cmd+Tab) is always accepted,
/// bypassing the guards for incidental focus changes: windows being
/// hidden by the WM and focus changes shortly after a close or minimize.
pub fn focus_action(
  is_application_activation: bool,
  is_hiding: bool,
  should_override_focus: bool,
) -> FocusAction {
  if is_application_activation {
    FocusAction::Accept
  } else if should_override_focus {
    FocusAction::RestorePrevious
  } else if is_hiding {
    FocusAction::Ignore
  } else {
    FocusAction::Accept
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn application_activation_bypasses_incidental_focus_guards() {
    assert_eq!(focus_action(true, true, true), FocusAction::Accept);
    assert_eq!(focus_action(true, false, true), FocusAction::Accept);
    assert_eq!(focus_action(true, true, false), FocusAction::Accept);
  }

  #[test]
  fn incidental_focus_changes_keep_existing_guards() {
    assert_eq!(
      focus_action(false, false, true),
      FocusAction::RestorePrevious
    );
    assert_eq!(focus_action(false, true, false), FocusAction::Ignore);
    assert_eq!(focus_action(false, false, false), FocusAction::Accept);
  }
}
