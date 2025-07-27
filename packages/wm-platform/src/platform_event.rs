use wm_common::Point;

use super::NativeWindow;

#[derive(Clone, Debug)]
pub enum PlatformEvent {
  Window(WindowEvent),
  Keyboard(KeybindingEvent),
  MouseMove(MouseMoveEvent),
  DisplaySettingsChanged,
}

#[derive(Clone, Debug)]
pub enum WindowEvent {
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

#[derive(Clone, Debug)]
pub struct KeybindingEvent {
  pub key: String,
  pub command: String,
  pub mode: String,
}

#[derive(Clone, Debug)]
pub struct MouseMoveEvent {
  /// Location of mouse with 0,0 being the top-left corner of the primary
  /// monitor.
  pub point: Point,

  /// Whether either left or right-click is currently pressed.
  pub is_mouse_down: bool,
}
