use super::NativeWindow;
use crate::{
  platform_impl::{
    MouseEventNotificationInner, WindowEventNotificationInner,
  },
  Keybinding, Point, WindowId,
};

#[derive(Clone, Debug)]
pub enum PlatformEvent {
  Window(WindowEvent),
  Keybinding(KeybindingEvent),
  Mouse(MouseEvent),
  DisplaySettingsChanged,
}

#[derive(Clone, Debug)]
pub enum WindowEvent {
  /// Window gained focus.
  Focused {
    window: NativeWindow,
    notification: WindowEventNotification,
  },

  /// Window was hidden.
  Hidden {
    window: NativeWindow,
    notification: WindowEventNotification,
  },

  /// Size or position of window has changed.
  ///
  /// `is_interactive_start` and `is_interactive_end` indicate whether the
  /// move or resize was initiated via manual interaction with the
  /// window's drag handles.
  ///
  /// # Platform-specific
  ///
  /// - **Windows**: Corresponds to `EVENT_OBJECT_LOCATIONCHANGE`,
  ///   `EVENT_SYSTEM_MOVESIZESTART`, and `EVENT_SYSTEM_MOVESIZEEND`.
  /// - **macOS**: Corresponds to `AXWindowMoved` and `AXWindowResized`.
  ///   The `is_interactive_start` and `is_interactive_end` flags are
  ///   always `false`.
  MovedOrResized {
    window: NativeWindow,
    is_interactive_start: bool,
    is_interactive_end: bool,
    notification: WindowEventNotification,
  },

  /// Window was minimized.
  Minimized {
    window: NativeWindow,
    notification: WindowEventNotification,
  },

  /// Window was restored from minimized state.
  MinimizeEnded {
    window: NativeWindow,
    notification: WindowEventNotification,
  },

  /// Window became visible.
  Shown {
    window: NativeWindow,
    notification: WindowEventNotification,
  },

  /// Window title changed.
  TitleChanged {
    window: NativeWindow,
    notification: WindowEventNotification,
  },

  /// Window was destroyed.
  Destroyed {
    window_id: WindowId,
    notification: WindowEventNotification,
  },
}

impl WindowEvent {
  /// Get the window handle if available (not available for
  /// `WindowEvent::Destroyed`).
  #[must_use]
  pub fn window(&self) -> Option<&NativeWindow> {
    match self {
      Self::Focused { window, .. }
      | Self::Hidden { window, .. }
      | Self::MovedOrResized { window, .. }
      | Self::Minimized { window, .. }
      | Self::MinimizeEnded { window, .. }
      | Self::Shown { window, .. }
      | Self::TitleChanged { window, .. } => Some(window),
      Self::Destroyed { .. } => None,
    }
  }

  /// Returns the platform-specific window event notification.
  pub fn notification(&self) -> &WindowEventNotification {
    match self {
      Self::Focused { notification, .. }
      | Self::Hidden { notification, .. }
      | Self::MovedOrResized { notification, .. }
      | Self::Minimized { notification, .. }
      | Self::MinimizeEnded { notification, .. }
      | Self::Shown { notification, .. }
      | Self::TitleChanged { notification, .. } => notification,
      Self::Destroyed { notification, .. } => notification,
    }
  }
}

/// Platform-specific window event notification.
///
/// Some events are "synthetic" and do not have a corresponding
/// notification (represented by `None`).
///
/// Synthetic events can occur when:
/// * On macOS, `WindowEvent::Shown` is emitted for new visible windows
///   even if a different notification is received first.
#[derive(Clone, Debug)]
pub struct WindowEventNotification(
  pub Option<WindowEventNotificationInner>,
);

#[derive(Clone, Debug)]
pub struct KeybindingEvent(pub Keybinding);

#[derive(Clone, Debug)]
pub enum MouseButton {
  Left,
  Right,
}

#[derive(Clone, Debug)]
pub enum MouseEvent {
  MouseMove {
    position: Point,
    pressed_buttons: Vec<MouseButton>,
    notification: MouseEventNotification,
  },
  MouseButtonDown {
    position: Point,
    button: MouseButton,
    pressed_buttons: Vec<MouseButton>,
    notification: MouseEventNotification,
  },
  MouseButtonUp {
    position: Point,
    button: MouseButton,
    pressed_buttons: Vec<MouseButton>,
    notification: MouseEventNotification,
  },
}

impl MouseEvent {
  /// Returns the position of the mouse.
  ///
  /// `0,0` is the top-left corner of the primary monitor.
  pub fn position(&self) -> &Point {
    match self {
      MouseEvent::MouseMove { position, .. } => position,
      MouseEvent::MouseButtonDown { position, .. } => position,
      MouseEvent::MouseButtonUp { position, .. } => position,
    }
  }

  /// Returns the buttons that are currently pressed.
  pub fn pressed_buttons(&self) -> &[MouseButton] {
    match self {
      MouseEvent::MouseMove {
        pressed_buttons, ..
      } => pressed_buttons,
      MouseEvent::MouseButtonDown {
        pressed_buttons, ..
      } => pressed_buttons,
      MouseEvent::MouseButtonUp {
        pressed_buttons, ..
      } => pressed_buttons,
    }
  }

  /// Returns the platform-specific mouse event notification.
  pub fn notification(&self) -> &MouseEventNotification {
    match self {
      MouseEvent::MouseMove { notification, .. } => notification,
      MouseEvent::MouseButtonDown { notification, .. } => notification,
      MouseEvent::MouseButtonUp { notification, .. } => notification,
    }
  }
}

/// Platform-specific mouse event notification.
#[derive(Clone, Debug)]
pub struct MouseEventNotification(pub MouseEventNotificationInner);
