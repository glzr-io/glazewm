use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use wm_macros::KeyConversions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, KeyConversions)]
#[key(win_prefix = windows::Win32::UI::Input::KeyboardAndMouse, macos_prefix = None)]
pub enum Key {
  #[key("a", win = VK_A, macos = None)]
  A,
  #[key("b", win = VK_B, macos = None)]
  B,
  #[key("c", win = VK_C, macos = None)]
  C,
  #[key("d", win = VK_D, macos = None)]
  D,
  #[key("e", win = VK_E, macos = None)]
  E,
  #[key("f", win = VK_F, macos = None)]
  F,
  #[key("g", win = VK_G, macos = None)]
  G,
  #[key("h", win = VK_H, macos = None)]
  H,
  #[key("i", win = VK_I, macos = None)]
  I,
  #[key("j", win = VK_J, macos = None)]
  J,
  #[key("k", win = VK_K, macos = None)]
  K,
  #[key("l", win = VK_L, macos = None)]
  L,
  #[key("m", win = VK_M, macos = None)]
  M,
  #[key("n", win = VK_N, macos = None)]
  N,
  #[key("o", win = VK_O, macos = None)]
  O,
  #[key("p", win = VK_P, macos = None)]
  P,
  #[key("q", win = VK_Q, macos = None)]
  Q,
  #[key("r", win = VK_R, macos = None)]
  R,
  #[key("s", win = VK_S, macos = None)]
  S,
  #[key("t", win = VK_T, macos = None)]
  T,
  #[key("u", win = VK_U, macos = None)]
  U,
  #[key("v", win = VK_V, macos = None)]
  V,
  #[key("w", win = VK_W, macos = None)]
  W,
  #[key("x", win = VK_X, macos = None)]
  X,
  #[key("y",  win = VK_Y, macos = None)]
  Y,
  #[key("z", win = VK_Z, macos = None)]
  Z,
  #[key("0", win = VK_0, macos = None)]
  D0,
  #[key("1", win = VK_1, macos = None)]
  D1,
  #[key("2", win = VK_2, macos = None)]
  D2,
  #[key("3", win = VK_3, macos = None)]
  D3,
  #[key("4", win = VK_4, macos = None)]
  D4,
  #[key("5", win = VK_5, macos = None)]
  D5,
  #[key("6", win = VK_6, macos = None)]
  D6,
  #[key("7", win = VK_7, macos = None)]
  D7,
  #[key("8", win = VK_8, macos = None)]
  D8,
  #[key("9", win = VK_9, macos = None)]
  D9,
  #[key("numpad 0", win = VK_NUMPAD0, macos = None)]
  Numpad0,
  #[key("numpad 1", win = VK_NUMPAD1, macos = None)]
  Numpad1,
  #[key("numpad 2", win = VK_NUMPAD2, macos = None)]
  Numpad2,
  #[key("numpad 3", win = VK_NUMPAD3, macos = None)]
  Numpad3,
  #[key("numpad 4", win = VK_NUMPAD4, macos = None)]
  Numpad4,
  #[key("numpad 5", win = VK_NUMPAD5, macos = None)]
  Numpad5,
  #[key("numpad 6", win = VK_NUMPAD6, macos = None)]
  Numpad6,
  #[key("numpad 7", win = VK_NUMPAD7, macos = None)]
  Numpad7,
  #[key("numpad 8", win = VK_NUMPAD8, macos = None)]
  Numpad8,
  #[key("numpad 9", win = VK_NUMPAD9, macos = None)]
  Numpad9,
  #[key("f1", win = VK_F1, macos = None)]
  F1,
  #[key("f2", win = VK_F2, macos = None)]
  F2,
  #[key("f3", win = VK_F3, macos = None)]
  F3,
  #[key("f4", win = VK_F4, macos = None)]
  F4,
  #[key("f5", win = VK_F5, macos = None)]
  F5,
  #[key("f6", win = VK_F6, macos = None)]
  F6,
  #[key("f7", win = VK_F7, macos = None)]
  F7,
  #[key("f8", win = VK_F8, macos = None)]
  F8,
  #[key("f9", win = VK_F9, macos = None)]
  F9,
  #[key("f10", win = VK_F10, macos = None)]
  F10,
  #[key("f11", win = VK_F11, macos = None)]
  F11,
  #[key("f12", win = VK_F12, macos = None)]
  F12,
  #[key("f13", win = VK_F13, macos = None)]
  F13,
  #[key("f14", win = VK_F14, macos = None)]
  F14,
  #[key("f15", win = VK_F15, macos = None)]
  F15,
  #[key("f16", win = VK_F16, macos = None)]
  F16,
  #[key("f17", win = VK_F17, macos = None)]
  F17,
  #[key("f18", win = VK_F18, macos = None)]
  F18,
  #[key("f19", win = VK_F19, macos = None)]
  F19,
  #[key("f20", win = VK_F20, macos = None)]
  F20,
  #[key("f21", win = VK_F21, macos = None)]
  F21,
  #[key("f22", win = VK_F22, macos = None)]
  F22,
  #[key("f23", win = VK_F23, macos = None)]
  F23,
  #[key("f24", win = VK_F24, macos = None)]
  F24,
  #[key("shift", win = VK_SHIFT, macos = None)]
  Shift,
  #[key("lshift", win = VK_LSHIFT, macos = None)]
  LShift,
  #[key("rshift", win = VK_RSHIFT, macos = None)]
  RShift,
  #[key("control" | "ctrl" | "control key", win = VK_CONTROL, macos = None)]
  Control,
  #[key("lcontrol" | "lctrl" | "lcontrol key", win = VK_LCONTROL, macos = None)]
  LControl,
  #[key("rcontrol" | "rctrl" | "rcontrol key", win = VK_RCONTROL, macos = None)]
  RControl,
  #[key("alt" | "menu", win = VK_MENU, macos = None)]
  Alt,
  #[key("lalt" | "lmenu",  win = VK_LMENU, macos = None)]
  LAlt,
  #[key("ralt" | "rmenu", win = VK_RMENU, macos = None)]
  RAlt,
  #[key("win",   win = Virt(VK_LWIN), macos = None)]
  Win,
  #[key("lwin",  win = VK_LWIN, macos = None)]
  LWin,
  #[key("rwin", win = VK_RWIN, macos = None)]
  RWin,
  #[key("space", win = VK_SPACE, macos = None)]
  Space,
  #[key("escape" | "esc", win = VK_ESCAPE, macos = None)]
  Escape,
  #[key("back", win = VK_BACK, macos = None)]
  Back,
  #[key("tab", win = VK_TAB, macos = None)]
  Tab,
  #[key("enter" | "return", win = VK_RETURN, macos = None)]
  Enter,
  #[key("left", win = VK_LEFT, macos = None)]
  Left,
  #[key("right", win = VK_RIGHT, macos = None)]
  Right,
  #[key("up", win = VK_UP, macos = None)]
  Up,
  #[key("down", win = VK_DOWN, macos = None)]
  Down,
  #[key("num lock", win = VK_NUMLOCK, macos = None)]
  NumLock,
  #[key("scroll lock", win = VK_SCROLL, macos = None)]
  ScrollLock,
  #[key("caps lock", win = VK_CAPITAL, macos = None)]
  CapsLock,
  #[key("page up", win = VK_PRIOR, macos = None)]
  PageUp,
  #[key("page down", win = VK_NEXT, macos = None)]
  PageDown,
  #[key("insert", win = VK_INSERT, macos = None)]
  Insert,
  #[key("delete" | "del", win = VK_DELETE, macos = None)]
  Delete,
  #[key("end", win = VK_END, macos = None)]
  End,
  #[key("home", win = VK_HOME, macos = None)]
  Home,
  #[key("print screen", win = VK_SNAPSHOT, macos = None)]
  PrintScreen,
  #[key("multiply", win = VK_MULTIPLY, macos = None)]
  Multiply,
  #[key("add", win = VK_ADD, macos = None)]
  Add,
  #[key("subtract", win = VK_SUBTRACT, macos = None)]
  Subtract,
  #[key("decimal", win = VK_DECIMAL, macos = None)]
  Decimal,
  #[key("divide", win = VK_DIVIDE, macos = None)]
  Divide,
  #[key("volume up", win = VK_VOLUME_UP, macos = None)]
  VolumeUp,
  #[key("volume down", win = VK_VOLUME_DOWN, macos = None)]
  VolumeDown,
  #[key("volume mute", win = VK_VOLUME_MUTE, macos = None)]
  VolumeMute,
  #[key("media next track", win = VK_MEDIA_NEXT_TRACK, macos = None)]
  MediaNextTrack,
  #[key("media prev track" | "media previous track", win = VK_MEDIA_PREV_TRACK, macos = None)]
  MediaPrevTrack,
  #[key("media stop", win = VK_MEDIA_STOP, macos = None)]
  MediaStop,
  #[key("media play pause", win = VK_MEDIA_PLAY_PAUSE, macos = None)]
  MediaPlayPause,
  #[key("oem semicolon", win = VK_OEM_1, macos = None)]
  OemSemicolon,
  #[key("oem question", win = VK_OEM_2, macos = None)]
  OemQuestion,
  #[key("oem tilde", win = VK_OEM_3, macos = None)]
  OemTilde,
  #[key("oem open brackets", win = VK_OEM_4, macos = None)]
  OemOpenBrackets,
  #[key("oem pipe", win = VK_OEM_5, macos = None)]
  OemPipe,
  #[key("oem close brackets", win = VK_OEM_6, macos = None)]
  OemCloseBrackets,
  #[key("oem quotes", win = VK_OEM_7, macos = None)]
  OemQuotes,
  #[key("oem plus", win = VK_OEM_PLUS, macos = None)]
  OemPlus,
  #[key("oem comma", win = VK_OEM_COMMA, macos = None)]
  OemComma,
  #[key("oem minus", win = VK_OEM_MINUS, macos = None)]
  OemMinus,
  #[key("oem period", win = VK_OEM_PERIOD, macos = None)]
  OemPeriod,
  #[key(..)]
  Custom(u16),
}

impl Key {
  /// Check if the key is analogous to another key.
  /// Returns `true` if the keys are the same or if `other` is a
  /// specialisation of self.
  ///
  /// # Example
  /// ```rs
  /// assert!(Key::A.is_analogous(Key::A));
  /// assert!(Key::Shift.is_analogous(Key::LShift));
  /// assert!(Key::LControl.is_analogous(Key::RControl) == false);
  /// ```
  pub fn is_analogous(self, other: Key) -> bool {
    #[allow(clippy::match_same_arms)]
    match (self, other) {
      (Key::Shift, Key::LShift | Key::RShift) => true,
      (Key::Control, Key::LControl | Key::RControl) => true,
      (Key::Alt, Key::LAlt | Key::RAlt) => true,
      (Key::Win, Key::LWin | Key::RWin) => true,
      _ => self == other,
    }
  }

  /// Returns whether the key is a generic key, such as `Shift`, `Control`,
  /// `Alt`, or `Win` instead of the more specific versions like
  /// `LShift`, `RShift`, etc.
  pub fn is_generic(self) -> bool {
    matches!(self, Key::Shift | Key::Control | Key::Alt | Key::Win)
  }

  /// Returns a generic version of the key.
  /// Special keys like `LShift`, `RShift`, `LControl`, `RControl` etc.
  /// will return the gereric Shift, Control, Alt, or Win key.
  /// Other keys will return themselves unchanged.
  pub fn get_generic(self) -> Self {
    match self {
      Key::LShift | Key::RShift => Key::Shift,
      Key::LControl | Key::RControl => Key::Control,
      Key::LAlt | Key::RAlt => Key::Alt,
      Key::LWin | Key::RWin => Key::Win,
      _ => self,
    }
  }

  /// Get the specific key(s) for a Key.
  /// Generic keys like `Shift`, `Control`, `Alt`, and `Win` will return
  /// both the left and right versions of the key.
  /// Non-generic keys will return a vector containing just the key itself.
  pub fn get_specifics(self) -> Vec<Key> {
    match self {
      Key::Shift => vec![Key::LShift, Key::RShift],
      Key::Control => vec![Key::LControl, Key::RControl],
      Key::Alt => vec![Key::LAlt, Key::RAlt],
      Key::Win => vec![Key::LWin, Key::RWin],
      _ => vec![self],
    }
  }

  /// Gets whether this key is currently down.
  pub fn is_down(self) -> bool {
    self.get_specifics().iter().any(|key| key.is_down_raw())
  }

  /// Gets whether this key is currently down using the raw key
  /// code.
  pub fn is_down_raw(self) -> bool {
    let vk_code = self.into_vk();
    unsafe { (GetKeyState(vk_code.into()) & 0x80) == 0x80 }
  }
}

#[cfg(test)]
mod tests {
  use wm_macros::KeyConversions;

  #[test]
  fn test_key_conversions_output() {
    #[derive(Debug, PartialEq, Eq, KeyConversions)]
    #[key(win_prefix = windows::Win32::UI::Input::KeyboardAndMouse, macos_prefix = None)]
    enum Key {
      #[key("a", win = VK_A, macos = None)]
      A,
      #[key("b" | "c", win = VK_B, macos = None)]
      BC,
      #[key("d e", win = VK_D, macos = None)]
      DE,
      #[key(..)]
      Custom(u16),
    }

    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_A, VK_B, VK_D};

    assert_eq!(Key::A.into_vk(), VK_A.0);
    assert_eq!(Key::BC.into_vk(), VK_B.0);
    assert_eq!(Key::DE.into_vk(), VK_D.0);

    assert_eq!(Key::from_str("a"), Some(Key::A));
    assert_eq!(Key::from_str("b"), Some(Key::BC));
    assert_eq!(Key::from_str("c"), Some(Key::BC));
    assert_eq!(Key::from_str("d e"), Some(Key::DE));
    assert_eq!(Key::from_str("d_e"), Some(Key::DE));
    assert_eq!(Key::from_str("d-e"), Some(Key::DE));
    assert_eq!(Key::from_str("dE"), Some(Key::DE));
    assert_eq!(Key::from_str("this_is_not_a_key"), None);

    assert_eq!(Key::from_vk(VK_A.0), Key::A);
    assert_eq!(Key::from_vk(VK_B.0), Key::BC);
    assert_eq!(Key::from_vk(VK_D.0), Key::DE);
  }
}
