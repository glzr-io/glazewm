use wm_common::Point;

use super::NativeWindow;
use crate::Keybinding;

#[derive(Clone, Debug)]
pub enum PlatformEvent {
  Window(WindowEvent),
  Keybinding(KeybindingEvent),
  MouseMove(MouseMoveEvent),
  DisplaySettingsChanged,
}

#[derive(Clone, Debug)]
pub enum WindowEvent {
  Focus(NativeWindow),
  Hide(NativeWindow),
  LocationChange(NativeWindow),
  Minimize(NativeWindow),
  MinimizeEnd(NativeWindow),
  MoveOrResizeEnd(NativeWindow),
  MoveOrResizeStart(NativeWindow),
  Show(NativeWindow),
  TitleChange(NativeWindow),
}

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
