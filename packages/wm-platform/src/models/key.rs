use std::{fmt, str::FromStr};

/// Cross-platform key representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
  // Letter keys
  A,
  B,
  C,
  D,
  E,
  F,
  G,
  H,
  I,
  J,
  K,
  L,
  M,
  N,
  O,
  P,
  Q,
  R,
  S,
  T,
  U,
  V,
  W,
  X,
  Y,
  Z,

  // Number keys
  D0,
  D1,
  D2,
  D3,
  D4,
  D5,
  D6,
  D7,
  D8,
  D9,

  // Function keys
  F1,
  F2,
  F3,
  F4,
  F5,
  F6,
  F7,
  F8,
  F9,
  F10,
  F11,
  F12,
  F13,
  F14,
  F15,
  F16,
  F17,
  F18,
  F19,
  F20,
  F21,
  F22,
  F23,
  F24,

  // Modifier keys
  Cmd,
  Ctrl,
  Alt,
  Shift,
  LCmd,
  RCmd,
  LCtrl,
  RCtrl,
  LAlt,
  RAlt,
  LShift,
  RShift,
  LWin,
  RWin,
  Win,

  // Special keys
  Space,
  Tab,
  Enter,
  Return,
  Delete,
  Escape,
  Backspace,

  // Arrow keys
  Left,
  Right,
  Up,
  Down,

  // Other keys
  Home,
  End,
  PageUp,
  PageDown,
  Insert,

  // Punctuation (common ones)
  Semicolon,
  Quote,
  Comma,
  Period,
  Slash,
  Backslash,
  LeftBracket,
  RightBracket,
  Minus,
  Equal,
  Grave,

  // Numpad
  Numpad0,
  Numpad1,
  Numpad2,
  Numpad3,
  Numpad4,
  Numpad5,
  Numpad6,
  Numpad7,
  Numpad8,
  Numpad9,
  NumpadAdd,
  NumpadSubtract,
  NumpadMultiply,
  NumpadDivide,
  NumpadDecimal,
}

impl FromStr for Key {
  type Err = KeyParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      // Letter keys
      "a" => Ok(Key::A),
      "b" => Ok(Key::B),
      "c" => Ok(Key::C),
      "d" => Ok(Key::D),
      "e" => Ok(Key::E),
      "f" => Ok(Key::F),
      "g" => Ok(Key::G),
      "h" => Ok(Key::H),
      "i" => Ok(Key::I),
      "j" => Ok(Key::J),
      "k" => Ok(Key::K),
      "l" => Ok(Key::L),
      "m" => Ok(Key::M),
      "n" => Ok(Key::N),
      "o" => Ok(Key::O),
      "p" => Ok(Key::P),
      "q" => Ok(Key::Q),
      "r" => Ok(Key::R),
      "s" => Ok(Key::S),
      "t" => Ok(Key::T),
      "u" => Ok(Key::U),
      "v" => Ok(Key::V),
      "w" => Ok(Key::W),
      "x" => Ok(Key::X),
      "y" => Ok(Key::Y),
      "z" => Ok(Key::Z),

      // Numbers
      "0" => Ok(Key::D0),
      "1" => Ok(Key::D1),
      "2" => Ok(Key::D2),
      "3" => Ok(Key::D3),
      "4" => Ok(Key::D4),
      "5" => Ok(Key::D5),
      "6" => Ok(Key::D6),
      "7" => Ok(Key::D7),
      "8" => Ok(Key::D8),
      "9" => Ok(Key::D9),

      // Function keys
      "f1" => Ok(Key::F1),
      "f2" => Ok(Key::F2),
      "f3" => Ok(Key::F3),
      "f4" => Ok(Key::F4),
      "f5" => Ok(Key::F5),
      "f6" => Ok(Key::F6),
      "f7" => Ok(Key::F7),
      "f8" => Ok(Key::F8),
      "f9" => Ok(Key::F9),
      "f10" => Ok(Key::F10),
      "f11" => Ok(Key::F11),
      "f12" => Ok(Key::F12),

      // Modifiers
      "cmd" | "command" => Ok(Key::Cmd),
      "ctrl" | "control" => Ok(Key::Ctrl),
      "alt" | "option" => Ok(Key::Alt),
      "shift" => Ok(Key::Shift),

      // Special keys
      "space" => Ok(Key::Space),
      "tab" => Ok(Key::Tab),
      "enter" | "return" => Ok(Key::Enter),
      "delete" => Ok(Key::Delete),
      "escape" => Ok(Key::Escape),
      "backspace" => Ok(Key::Backspace),

      // Arrow keys
      "left" => Ok(Key::Left),
      "right" => Ok(Key::Right),
      "up" => Ok(Key::Up),
      "down" => Ok(Key::Down),

      // Punctuation
      ";" => Ok(Key::Semicolon),
      "'" => Ok(Key::Quote),
      "," => Ok(Key::Comma),
      "." => Ok(Key::Period),
      "/" => Ok(Key::Slash),
      "\\" => Ok(Key::Backslash),
      "[" => Ok(Key::LeftBracket),
      "]" => Ok(Key::RightBracket),
      "-" => Ok(Key::Minus),
      "=" => Ok(Key::Equal),
      "`" => Ok(Key::Grave),

      _ => Err(KeyParseError::UnknownKey(s.to_string())),
    }
  }
}

impl fmt::Display for Key {
  #[allow(clippy::too_many_lines)]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      Key::A => "A",
      Key::B => "B",
      Key::C => "C",
      Key::D => "D",
      Key::E => "E",
      Key::F => "F",
      Key::G => "G",
      Key::H => "H",
      Key::I => "I",
      Key::J => "J",
      Key::K => "K",
      Key::L => "L",
      Key::M => "M",
      Key::N => "N",
      Key::O => "O",
      Key::P => "P",
      Key::Q => "Q",
      Key::R => "R",
      Key::S => "S",
      Key::T => "T",
      Key::U => "U",
      Key::V => "V",
      Key::W => "W",
      Key::X => "X",
      Key::Y => "Y",
      Key::Z => "Z",

      Key::D0 => "0",
      Key::D1 => "1",
      Key::D2 => "2",
      Key::D3 => "3",
      Key::D4 => "4",
      Key::D5 => "5",
      Key::D6 => "6",
      Key::D7 => "7",
      Key::D8 => "8",
      Key::D9 => "9",

      Key::F1 => "F1",
      Key::F2 => "F2",
      Key::F3 => "F3",
      Key::F4 => "F4",
      Key::F5 => "F5",
      Key::F6 => "F6",
      Key::F7 => "F7",
      Key::F8 => "F8",
      Key::F9 => "F9",
      Key::F10 => "F10",
      Key::F11 => "F11",
      Key::F12 => "F12",
      Key::F13 => "F13",
      Key::F14 => "F14",
      Key::F15 => "F15",
      Key::F16 => "F16",
      Key::F17 => "F17",
      Key::F18 => "F18",
      Key::F19 => "F19",
      Key::F20 => "F20",
      Key::F21 => "F21",
      Key::F22 => "F22",
      Key::F23 => "F23",
      Key::F24 => "F24",

      Key::Cmd => "Cmd",
      Key::Ctrl => "Ctrl",
      Key::Alt => "Alt",
      Key::Shift => "Shift",
      Key::LCmd => "LCmd",
      Key::RCmd => "RCmd",
      Key::LCtrl => "LCtrl",
      Key::RCtrl => "RCtrl",
      Key::LAlt => "LAlt",
      Key::RAlt => "RAlt",
      Key::LShift => "LShift",
      Key::RShift => "RShift",
      Key::LWin => "LWin",
      Key::RWin => "RWin",
      Key::Win => "Win",

      Key::Space => "Space",
      Key::Tab => "Tab",
      Key::Enter => "Enter",
      Key::Return => "Return",
      Key::Delete => "Delete",
      Key::Escape => "Escape",
      Key::Backspace => "Backspace",

      Key::Left => "Left",
      Key::Right => "Right",
      Key::Up => "Up",
      Key::Down => "Down",

      Key::Home => "Home",
      Key::End => "End",
      Key::PageUp => "PageUp",
      Key::PageDown => "PageDown",
      Key::Insert => "Insert",

      Key::Semicolon => ";",
      Key::Quote => "'",
      Key::Comma => ",",
      Key::Period => ".",
      Key::Slash => "/",
      Key::Backslash => "\\",
      Key::LeftBracket => "[",
      Key::RightBracket => "]",
      Key::Minus => "-",
      Key::Equal => "=",
      Key::Grave => "`",

      Key::Numpad0 => "Numpad0",
      Key::Numpad1 => "Numpad1",
      Key::Numpad2 => "Numpad2",
      Key::Numpad3 => "Numpad3",
      Key::Numpad4 => "Numpad4",
      Key::Numpad5 => "Numpad5",
      Key::Numpad6 => "Numpad6",
      Key::Numpad7 => "Numpad7",
      Key::Numpad8 => "Numpad8",
      Key::Numpad9 => "Numpad9",
      Key::NumpadAdd => "NumpadAdd",
      Key::NumpadSubtract => "NumpadSubtract",
      Key::NumpadMultiply => "NumpadMultiply",
      Key::NumpadDivide => "NumpadDivide",
      Key::NumpadDecimal => "NumpadDecimal",
    };

    write!(f, "{s}")
  }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyParseError {
  #[error("Unknown key: {0}")]
  UnknownKey(String),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_key_parsing() {
    assert_eq!("a".parse::<Key>().unwrap(), Key::A);
    assert_eq!("cmd".parse::<Key>().unwrap(), Key::Cmd);
    assert_eq!("f1".parse::<Key>().unwrap(), Key::F1);
    assert_eq!("space".parse::<Key>().unwrap(), Key::Space);
    assert_eq!("enter".parse::<Key>().unwrap(), Key::Enter);
    assert_eq!("return".parse::<Key>().unwrap(), Key::Enter);

    assert!("invalid".parse::<Key>().is_err());
  }

  #[test]
  fn test_key_display() {
    assert_eq!(Key::A.to_string(), "A");
    assert_eq!(Key::Cmd.to_string(), "Cmd");
    assert_eq!(Key::F1.to_string(), "F1");
    assert_eq!(Key::Space.to_string(), "Space");
    assert_eq!(Key::Semicolon.to_string(), ";");
  }

  #[test]
  fn test_key_parsing_case_insensitive() {
    assert_eq!("A".parse::<Key>().unwrap(), Key::A);
    assert_eq!("CMD".parse::<Key>().unwrap(), Key::Cmd);
    assert_eq!("F1".parse::<Key>().unwrap(), Key::F1);
    assert_eq!("SPACE".parse::<Key>().unwrap(), Key::Space);
  }

  #[test]
  fn test_key_parsing_aliases() {
    assert_eq!("command".parse::<Key>().unwrap(), Key::Cmd);
    assert_eq!("control".parse::<Key>().unwrap(), Key::Ctrl);
    assert_eq!("option".parse::<Key>().unwrap(), Key::Alt);
    assert_eq!("return".parse::<Key>().unwrap(), Key::Enter);
  }
}
