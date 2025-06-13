use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use wm_macros::KeyConversions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, KeyConversions)]
pub enum Key {
  #[key("a", VK_A)]
  A,
  #[key("b", VK_B)]
  B,
  #[key("c", VK_C)]
  C,
  #[key("d", VK_D)]
  D,
  #[key("e", VK_F)]
  E,
  #[key("f", VK_F)]
  F,
  #[key("g", VK_G)]
  G,
  #[key("h", VK_H)]
  H,
  #[key("i", VK_I)]
  I,
  #[key("j", VK_J)]
  J,
  #[key("k", VK_K)]
  K,
  #[key("l", VK_L)]
  L,
  #[key("m", VK_M)]
  M,
  #[key("n", VK_N)]
  N,
  #[key("o", VK_O)]
  O,
  #[key("p", VK_P)]
  P,
  #[key("q", VK_Q)]
  Q,
  #[key("r", VK_R)]
  R,
  #[key("s", VK_S)]
  S,
  #[key("t", VK_T)]
  T,
  #[key("u", VK_U)]
  U,
  #[key("v", VK_V)]
  V,
  #[key("w", VK_W)]
  W,
  #[key("x", VK_X)]
  X,
  #[key("y", VK_Y)]
  Y,
  #[key("z", VK_Z)]
  Z,
  #[key("0", VK_0)]
  D0,
  #[key("1", VK_1)]
  D1,
  #[key("2", VK_2)]
  D2,
  #[key("3", VK_3)]
  D3,
  #[key("4", VK_4)]
  D4,
  #[key("5", VK_5)]
  D5,
  #[key("6", VK_6)]
  D6,
  #[key("7", VK_7)]
  D7,
  #[key("8", VK_8)]
  D8,
  #[key("9", VK_9)]
  D9,
  #[key("numpad 0", VK_NUMPAD0)]
  Numpad0,
  #[key("numpad 1", VK_NUMPAD1)]
  Numpad1,
  #[key("numpad 2", VK_NUMPAD2)]
  Numpad2,
  #[key("numpad 3", VK_NUMPAD3)]
  Numpad3,
  #[key("numpad 4", VK_NUMPAD4)]
  Numpad4,
  #[key("numpad 5", VK_NUMPAD5)]
  Numpad5,
  #[key("numpad 6", VK_NUMPAD6)]
  Numpad6,
  #[key("numpad 7", VK_NUMPAD7)]
  Numpad7,
  #[key("numpad 8", VK_NUMPAD8)]
  Numpad8,
  #[key("numpad 9", VK_NUMPAD9)]
  Numpad9,
  #[key("f1", VK_F1)]
  F1,
  #[key("f2", VK_F2)]
  F2,
  #[key("f3", VK_F3)]
  F3,
  #[key("f4", VK_F4)]
  F4,
  #[key("f5", VK_F5)]
  F5,
  #[key("f6", VK_F6)]
  F6,
  #[key("f7", VK_F7)]
  F7,
  #[key("f8", VK_F8)]
  F8,
  #[key("f9", VK_F9)]
  F9,
  #[key("f10", VK_F10)]
  F10,
  #[key("f11", VK_F11)]
  F11,
  #[key("f12", VK_F12)]
  F12,
  #[key("f13", VK_F13)]
  F13,
  #[key("f14", VK_F14)]
  F14,
  #[key("f15", VK_F15)]
  F15,
  #[key("f16", VK_F16)]
  F16,
  #[key("f17", VK_F17)]
  F17,
  #[key("f18", VK_F18)]
  F18,
  #[key("f19", VK_F19)]
  F19,
  #[key("f20", VK_F20)]
  F20,
  #[key("f21", VK_F21)]
  F21,
  #[key("f22", VK_F22)]
  F22,
  #[key("f23", VK_F23)]
  F23,
  #[key("f24", VK_F24)]
  F24,
  #[key("shift", VK_SHIFT)]
  Shift,
  #[key("lshift", VK_LSHIFT)]
  LShift,
  #[key("rshift", VK_RSHIFT)]
  RShift,
  #[key("control" | "ctrl" | "control key", VK_CONTROL)]
  Control,
  #[key("lcontrol" | "lctrl" | "lcontrol key", VK_LCONTROL)]
  LControl,
  #[key("rcontrol" | "rctrl" | "rcontrol key", VK_RCONTROL)]
  RControl,
  #[key("alt" | "menu", VK_MENU)]
  Alt,
  #[key("lalt" | "lmenu", VK_LMENU)]
  LAlt,
  #[key("ralt" | "rmenu", VK_RMENU)]
  RAlt,
  #[key("win", VK_LWIN)]
  Win,
  #[key("lwin", VK_LWIN)]
  LWin,
  #[key("rwin", VK_RWIN)]
  RWin,
  #[key("space", VK_SPACE)]
  Space,
  #[key("escape" | "esc", VK_ESCAPE)]
  Escape,
  #[key("back", VK_BACK)]
  Back,
  #[key("tab", VK_TAB)]
  Tab,
  #[key("enter" | "return", VK_RETURN)]
  Enter,
  #[key("left", VK_LEFT)]
  Left,
  #[key("right", VK_RIGHT)]
  Right,
  #[key("up", VK_UP)]
  Up,
  #[key("down", VK_DOWN)]
  Down,
  #[key("num lock", VK_NUMLOCK)]
  NumLock,
  #[key("scroll lock", VK_SCROLL)]
  ScrollLock,
  #[key("caps lock", VK_CAPITAL)]
  CapsLock,
  #[key("page up", VK_PRIOR)]
  PageUp,
  #[key("page down", VK_NEXT)]
  PageDown,
  #[key("insert", VK_INSERT)]
  Insert,
  #[key("delete" | "del", VK_DELETE)]
  Delete,
  #[key("end", VK_END)]
  End,
  #[key("home", VK_HOME)]
  Home,
  #[key("print screen", VK_SNAPSHOT)]
  PrintScreen,
  #[key("multiply", VK_MULTIPLY)]
  Multiply,
  #[key("add", VK_ADD)]
  Add,
  #[key("subtract", VK_SUBTRACT)]
  Subtract,
  #[key("decimal", VK_DECIMAL)]
  Decimal,
  #[key("divide", VK_DIVIDE)]
  Divide,
  #[key("volume up", VK_VOLUME_UP)]
  VolumeUp,
  #[key("volume down", VK_VOLUME_DOWN)]
  VolumeDown,
  #[key("volume mute", VK_VOLUME_MUTE)]
  VolumeMute,
  #[key("media next track", VK_MEDIA_NEXT_TRACK)]
  MediaNextTrack,
  #[key("media prev track" | "media previous track", VK_MEDIA_PREV_TRACK)]
  MediaPrevTrack,
  #[key("media stop", VK_MEDIA_STOP)]
  MediaStop,
  #[key("media play pause", VK_MEDIA_PLAY_PAUSE)]
  MediaPlayPause,
  #[key("oem semicolon", VK_OEM_1)]
  OemSemicolon,
  #[key("oem question", VK_OEM_2)]
  OemQuestion,
  #[key("oem tilde", VK_OEM_3)]
  OemTilde,
  #[key("oem open brackets", VK_OEM_4)]
  OemOpenBrackets,
  #[key("oem pipe", VK_OEM_5)]
  OemPipe,
  #[key("oem close brackets", VK_OEM_6)]
  OemCloseBrackets,
  #[key("oem quotes", VK_OEM_7)]
  OemQuotes,
  #[key("oem plus", VK_OEM_PLUS)]
  OemPlus,
  #[key("oem comma", VK_OEM_COMMA)]
  OemComma,
  #[key("oem minus", VK_OEM_MINUS)]
  OemMinus,
  #[key("oem period", VK_OEM_PERIOD)]
  OemPeriod,
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
    enum Key {
      #[key("a", VK_A)]
      A,
      #[key("b" | "c", VK_B)]
      BC,
      #[key("d e", VK_D)]
      DE,
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
