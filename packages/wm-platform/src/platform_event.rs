use super::NativeWindow;
use crate::{Keybinding, Point, WindowId};

#[derive(Clone, Debug)]
pub enum PlatformEvent {
  Window(WindowEvent),
  Keybinding(KeybindingEvent),
  MouseMove(MouseMoveEvent),
  DisplaySettingsChanged,
}

#[derive(Clone, Debug)]
pub enum WindowEvent {
  /// Window gained focus.
  Focus {
    window: NativeWindow,
    notification: WindowEventNotification,
  },

  /// Window was hidden.
  Hide {
    window: NativeWindow,
    notification: WindowEventNotification,
  },

  /// Size or position of window has changed.
  ///
  /// # Platform-specific
  ///
  /// - **Windows**: Corresponds to `EVENT_OBJECT_LOCATIONCHANGE`,
  ///   `EVENT_SYSTEM_MOVESIZESTART`, and `EVENT_SYSTEM_MOVESIZEEND`.
  /// - **macOS**: Corresponds to `AXWindowMoved` and `AXWindowResized`.
  ///   The `is_interactive_start` and `is_interactive_end` flags are
  ///   always `false`.
  MoveOrResize {
    window: NativeWindow,
    is_interactive_start: bool,
    is_interactive_end: bool,
    notification: WindowEventNotification,
  },

  /// Window was minimized.
  Minimize {
    window: NativeWindow,
    notification: WindowEventNotification,
  },

  /// Window was restored from minimized state.
  MinimizeEnd {
    window: NativeWindow,
    notification: WindowEventNotification,
  },

  /// Window became visible.
  Show {
    window: NativeWindow,
    notification: WindowEventNotification,
  },

  /// Window title changed.
  TitleChange {
    window: NativeWindow,
    notification: WindowEventNotification,
  },

  /// Window was destroyed.
  Destroy {
    window_id: WindowId,
    notification: WindowEventNotification,
  },
}

impl WindowEvent {
  /// Get the window handle if available (not available for
  /// `WindowEvent::Destroy`).
  #[must_use]
  pub fn window(&self) -> Option<&NativeWindow> {
    match self {
      Self::Focus { window, .. }
      | Self::Hide { window, .. }
      | Self::MoveOrResize { window, .. }
      | Self::Minimize { window, .. }
      | Self::MinimizeEnd { window, .. }
      | Self::Show { window, .. }
      | Self::TitleChange { window, .. } => Some(window),
      Self::Destroy { .. } => None,
    }
  }
}

/// Platform-specific window event notification.
///
/// Some events are "synthetic" and do not have a corresponding
/// notification (represented by `None`).
///
/// Synthetic events can occur when:
/// * On macOS, `WindowEvent::Show` is emitted for new visible windows even
///   if a different notification is received first.
#[derive(Clone, Debug)]
pub struct WindowEventNotification(
  pub(crate) Option<WindowEventNotificationInner>,
);

impl WindowEventNotification {
  #[must_use]
  pub fn inner(&self) -> Option<&WindowEventNotificationInner> {
    self.0.as_ref()
  }
}

/// Windows-specific window event notification.
#[derive(Clone, Debug)]
#[cfg(target_os = "windows")]
pub struct WindowEventNotificationInner;

/// macOS-specific window event notification.
#[derive(Clone, Debug)]
#[cfg(target_os = "macos")]
pub struct WindowEventNotificationInner {
  /// Name of the notification (e.g. `AXWindowMoved`).
  pub name: String,

  /// Pointer to the `AXUIElement` that triggered the notification.
  pub ax_element_ptr: *mut std::ffi::c_void,
}

unsafe impl Send for WindowEventNotificationInner {}

#[derive(Clone, Debug)]
pub struct KeybindingEvent(pub Keybinding);

#[derive(Clone, Debug)]
pub struct MouseMoveEvent {
  /// Location of mouse with 0,0 being the top-left corner of the primary
  /// monitor.
  pub point: Point,

  /// Whether either left or right-click is currently pressed.
  pub is_mouse_down: bool,
}
