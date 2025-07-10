use wm_common::{KeybindingConfig, Point};

use crate::NativeWindow;

#[derive(Debug, Clone)]
pub enum DisplayEvent {
  DisplaySettingsChanged,
}

#[derive(Debug, Clone)]
pub enum KeyboardEventType {
  KeybindingTriggered,
}

#[derive(Debug, Clone)]
pub enum KeyboardEvent {
  KeybindingTriggered(KeybindingConfig),
}

#[derive(Debug, Clone)]
pub enum MouseEventType {
  MouseMove,
}

#[derive(Debug, Clone)]
pub struct MouseMoveEvent {
  /// Location of mouse with 0,0 being the top-left corner of the primary
  /// monitor.
  pub point: Point,

  /// Whether either left or right-click is currently pressed.
  pub is_mouse_down: bool,
}

#[derive(Debug, Clone)]
pub enum MouseEvent {
  MouseMove(MouseMoveEvent),
}

bitflags::bitflags! {
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowEventType: u16 {
    const WindowDestroyed =           0b0000_0000_0001;
    const WindowFocused =             0b0000_0000_0010;
    const WindowHidden =              0b0000_0000_0100;
    const WindowCloaked =             0b0000_0000_1000;
    const WindowLocationChanged =     0b0000_0001_0000;
    const WindowMinimized =           0b0000_0010_0000;
    const WindowMinimizeEnded =       0b0000_0100_0000;
    const WindowMovedOrResizedEnd =   0b0000_1000_0000;
    const WindowMovedOrResizedStart = 0b0001_0000_0000;
    const WindowShown =               0b0010_0000_0000;
    const WindowUncloaked =           0b0100_0000_0000;
    const WindowTitleChanged =        0b1000_0000_0000;
  }
}

impl WindowEventType {
  #[cfg(target_os = "windows")]
  #[must_use]
  pub fn id(&self) -> Vec<u32> {
    use windows::Win32::UI::WindowsAndMessaging::{
      EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE,
      EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_NAMECHANGE,
      EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED, EVENT_SYSTEM_FOREGROUND,
      EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART,
      EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART,
    };

    match self {
      WindowEventType::WindowDestroyed => EVENT_OBJECT_DESTROY,
      WindowEventType::WindowFocused => EVENT_SYSTEM_FOREGROUND,
      WindowEventType::WindowHidden => EVENT_OBJECT_HIDE,
      WindowEventType::WindowCloaked => EVENT_OBJECT_CLOAKED,
      WindowEventType::WindowLocationChanged => {
        EVENT_OBJECT_LOCATIONCHANGE
      }
      WindowEventType::WindowMinimized => EVENT_SYSTEM_MINIMIZESTART,
      WindowEventType::WindowMinimizeEnded => EVENT_SYSTEM_MINIMIZEEND,
      WindowEventType::WindowMovedOrResizedEnd => EVENT_SYSTEM_MOVESIZEEND,
      WindowEventType::WindowMovedOrResizedStart => {
        EVENT_SYSTEM_MOVESIZESTART
      }
      WindowEventType::WindowShown => EVENT_OBJECT_SHOW,
      WindowEventType::WindowUncloaked => EVENT_OBJECT_UNCLOAKED,
      WindowEventType::WindowTitleChanged => EVENT_OBJECT_NAMECHANGE,
    }
  }
}

#[derive(Debug, Clone)]
pub enum WindowEvent {
  WindowDestroyed(NativeWindow),
  WindowFocused(NativeWindow),
  WindowHidden(NativeWindow),
  WindowLocationChanged(NativeWindow),
  WindowMinimized(NativeWindow),
  WindowMinimizeEnded(NativeWindow),
  WindowMovedOrResizedEnd(NativeWindow),
  WindowMovedOrResizedStart(NativeWindow),
  WindowShown(NativeWindow),
  WindowTitleChanged(NativeWindow),
}

impl WindowEvent {
  #[must_use]
  pub fn get_type(&self) -> WindowEventType {
    match self {
      WindowEvent::WindowDestroyed(_) => WindowEventType::WindowDestroyed,
      WindowEvent::WindowFocused(_) => WindowEventType::WindowFocused,
      WindowEvent::WindowHidden(_) => WindowEventType::WindowHidden,
      WindowEvent::WindowLocationChanged(_) => {
        WindowEventType::WindowLocationChanged
      }
      WindowEvent::WindowMinimized(_) => WindowEventType::WindowMinimized,
      WindowEvent::WindowMinimizeEnded(_) => {
        WindowEventType::WindowMinimizeEnded
      }
      WindowEvent::WindowMovedOrResizedEnd(_) => {
        WindowEventType::WindowMovedOrResizedEnd
      }
      WindowEvent::WindowMovedOrResizedStart(_) => {
        WindowEventType::WindowMovedOrResizedStart
      }
      WindowEvent::WindowShown(_) => WindowEventType::WindowShown,
      WindowEvent::WindowTitleChanged(_) => {
        WindowEventType::WindowTitleChanged
      }
    }
  }
}
