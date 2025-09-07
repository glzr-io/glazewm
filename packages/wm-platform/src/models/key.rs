use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// Platform-specific keyboard key code.
///
/// Represents the raw key code from the underlying platform's keyboard
/// API.
///
/// # Platform-specific
///
/// - **Windows**: `u16` (Virtual key code from Windows API). See <https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes>
/// - **macOS**: `i64` (Virtual key code from `CGEvent`). See <https://developer.apple.com/documentation/coregraphics/cgeventfield/keyboardeventkeycode>
#[derive(
  Debug,
  Copy,
  Clone,
  PartialEq,
  Eq,
  PartialOrd,
  Ord,
  Hash,
  Serialize,
  Deserialize,
)]
pub struct KeyCode(
  #[cfg(target_os = "windows")] pub(crate) u16,
  #[cfg(target_os = "macos")] pub(crate) i64,
);

impl fmt::Display for KeyCode {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyParseError {
  #[error("Unknown key: {0}")]
  UnknownKey(String),
}

// TODO: Simplify key conversions from string with serde serialization. Use
// aliases for keys that can be defined in multiple ways.

/// Cross-platform key representation.
#[derive(
  Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
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
  #[serde(alias = "0")]
  D0,
  #[serde(alias = "1")]
  D1,
  #[serde(alias = "2")]
  D2,
  #[serde(alias = "3")]
  D3,
  #[serde(alias = "4")]
  D4,
  #[serde(alias = "5")]
  D5,
  #[serde(alias = "6")]
  D6,
  #[serde(alias = "7")]
  D7,
  #[serde(alias = "8")]
  D8,
  #[serde(alias = "9")]
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
  Win,
  #[serde(rename = "lcmd")]
  LCmd,
  #[serde(rename = "rcmd")]
  RCmd,
  #[serde(rename = "lctrl")]
  LCtrl,
  #[serde(rename = "rctrl")]
  RCtrl,
  #[serde(rename = "lalt")]
  LAlt,
  #[serde(rename = "ralt")]
  RAlt,
  #[serde(rename = "lshift")]
  LShift,
  #[serde(rename = "rshift")]
  RShift,
  #[serde(rename = "lwin")]
  LWin,
  #[serde(rename = "rwin")]
  RWin,

  // Special keys
  Space,
  Tab,
  Enter,
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
  #[serde(alias = "add")]
  NumpadAdd,
  #[serde(alias = "subtract")]
  NumpadSubtract,
  #[serde(alias = "multiply")]
  NumpadMultiply,
  #[serde(alias = "divide")]
  NumpadDivide,
  #[serde(alias = "decimal")]
  NumpadDecimal,
  NumLock,
  ScrollLock,
  CapsLock,

  // Media keys
  VolumeUp,
  VolumeDown,
  VolumeMute,
  MediaNextTrack,
  MediaPrevTrack,
  MediaStop,
  MediaPlayPause,
  PrintScreen,

  // OEM keys
  OemSemicolon,
  OemQuestion,
  OemTilde,
  OemOpenBrackets,
  OemPipe,
  OemCloseBrackets,
  OemQuotes,
  OemPlus,
  OemComma,
  OemMinus,
  OemPeriod,

  // Raw key codes that can't be mapped to a known key.
  Raw(KeyCode),
}

impl FromStr for Key {
  type Err = KeyParseError;

  #[allow(clippy::too_many_lines)]
  fn from_str(key_str: &str) -> Result<Self, Self::Err> {
    match key_str.to_lowercase().as_str() {
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
      "0" | "d0" => Ok(Key::D0),
      "1" | "d1" => Ok(Key::D1),
      "2" | "d2" => Ok(Key::D2),
      "3" | "d3" => Ok(Key::D3),
      "4" | "d4" => Ok(Key::D4),
      "5" | "d5" => Ok(Key::D5),
      "6" | "d6" => Ok(Key::D6),
      "7" | "d7" => Ok(Key::D7),
      "8" | "d8" => Ok(Key::D8),
      "9" | "d9" => Ok(Key::D9),
      "numpad0" => Ok(Key::Numpad0),
      "numpad1" => Ok(Key::Numpad1),
      "numpad2" => Ok(Key::Numpad2),
      "numpad3" => Ok(Key::Numpad3),
      "numpad4" => Ok(Key::Numpad4),
      "numpad5" => Ok(Key::Numpad5),
      "numpad6" => Ok(Key::Numpad6),
      "numpad7" => Ok(Key::Numpad7),
      "numpad8" => Ok(Key::Numpad8),
      "numpad9" => Ok(Key::Numpad9),
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
      "f13" => Ok(Key::F13),
      "f14" => Ok(Key::F14),
      "f15" => Ok(Key::F15),
      "f16" => Ok(Key::F16),
      "f17" => Ok(Key::F17),
      "f18" => Ok(Key::F18),
      "f19" => Ok(Key::F19),
      "f20" => Ok(Key::F20),
      "f21" => Ok(Key::F21),
      "f22" => Ok(Key::F22),
      "f23" => Ok(Key::F23),
      "f24" => Ok(Key::F24),
      "shift" | "shiftkey" => Ok(Key::Shift),
      "lshift" | "lshiftkey" => Ok(Key::LShift),
      "rshift" | "rshiftkey" => Ok(Key::RShift),
      "ctrl" | "controlkey" | "control" => Ok(Key::Ctrl),
      "lctrl" | "lcontrolkey" => Ok(Key::LCtrl),
      "rctrl" | "rcontrolkey" => Ok(Key::RCtrl),
      "alt" | "menu" => Ok(Key::Alt),
      "lalt" | "lmenu" => Ok(Key::LAlt),
      "ralt" | "rmenu" => Ok(Key::RAlt),
      "win" => Ok(Key::Win),
      "lwin" => Ok(Key::LWin),
      "rwin" => Ok(Key::RWin),
      "space" => Ok(Key::Space),
      "escape" => Ok(Key::Escape),
      "back" => Ok(Key::Backspace),
      "tab" => Ok(Key::Tab),
      "enter" | "return" => Ok(Key::Enter),
      "left" => Ok(Key::Left),
      "right" => Ok(Key::Right),
      "up" => Ok(Key::Up),
      "down" => Ok(Key::Down),
      "num_lock" => Ok(Key::NumLock),
      "scroll_lock" => Ok(Key::ScrollLock),
      "caps_lock" => Ok(Key::CapsLock),
      "page_up" => Ok(Key::PageUp),
      "page_down" => Ok(Key::PageDown),
      "insert" => Ok(Key::Insert),
      "delete" => Ok(Key::Delete),
      "end" => Ok(Key::End),
      "home" => Ok(Key::Home),
      "print_screen" => Ok(Key::PrintScreen),
      "multiply" => Ok(Key::NumpadMultiply),
      "add" => Ok(Key::NumpadAdd),
      "subtract" => Ok(Key::NumpadSubtract),
      "decimal" => Ok(Key::NumpadDecimal),
      "divide" => Ok(Key::NumpadDivide),
      "volume_up" => Ok(Key::VolumeUp),
      "volume_down" => Ok(Key::VolumeDown),
      "volume_mute" => Ok(Key::VolumeMute),
      "media_next_track" => Ok(Key::MediaNextTrack),
      "media_prev_track" => Ok(Key::MediaPrevTrack),
      "media_stop" => Ok(Key::MediaStop),
      "media_play_pause" => Ok(Key::MediaPlayPause),
      "oem_semicolon" => Ok(Key::OemSemicolon),
      "oem_question" => Ok(Key::OemQuestion),
      "oem_tilde" => Ok(Key::OemTilde),
      "oem_open_brackets" => Ok(Key::OemOpenBrackets),
      "oem_pipe" => Ok(Key::OemPipe),
      "oem_close_brackets" => Ok(Key::OemCloseBrackets),
      "oem_quotes" => Ok(Key::OemQuotes),
      "oem_plus" => Ok(Key::OemPlus),
      "oem_comma" => Ok(Key::OemComma),
      "oem_minus" => Ok(Key::OemMinus),
      "oem_period" => Ok(Key::OemPeriod),

      _ => {
        #[cfg(target_os = "macos")]
        {
          Err(KeyParseError::UnknownKey(key_str.to_string()))
        }
        #[cfg(target_os = "windows")]
        {
          use windows::Win32::UI::Input::KeyboardAndMouse::{
            GetKeyState, GetKeyboardLayout, VkKeyScanExW,
          };

          // Check if the key exists on the current keyboard layout.
          let utf16_key = key_str.encode_utf16().next()?;
          let layout = unsafe { GetKeyboardLayout(0) };
          let vk_code = unsafe { VkKeyScanExW(utf16_key, layout) };

          if vk_code == -1 {
            return Err(KeyParseError::UnknownKey(key_str.to_string()));
          }

          // The low-order byte contains the virtual-key code and the high-
          // order byte contains the shift state.
          let [high_order, low_order] = vk_code.to_be_bytes();

          // Key is valid if it doesn't require shift or alt to be pressed.
          match high_order {
            0 => Ok(Key::Raw(KeyCode(u16::from(low_order)))),
            _ => Err(KeyParseError::UnknownKey(key_str.to_string())),
          }
        }
      }
    }
  }
}

impl fmt::Display for Key {
  #[allow(clippy::too_many_lines)]
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      Key::A => "a",
      Key::B => "b",
      Key::C => "c",
      Key::D => "d",
      Key::E => "e",
      Key::F => "f",
      Key::G => "g",
      Key::H => "h",
      Key::I => "i",
      Key::J => "j",
      Key::K => "k",
      Key::L => "l",
      Key::M => "m",
      Key::N => "n",
      Key::O => "o",
      Key::P => "p",
      Key::Q => "q",
      Key::R => "r",
      Key::S => "s",
      Key::T => "t",
      Key::U => "u",
      Key::V => "v",
      Key::W => "w",
      Key::X => "x",
      Key::Y => "y",
      Key::Z => "z",

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

      Key::F1 => "f1",
      Key::F2 => "f2",
      Key::F3 => "f3",
      Key::F4 => "f4",
      Key::F5 => "f5",
      Key::F6 => "f6",
      Key::F7 => "f7",
      Key::F8 => "f8",
      Key::F9 => "f9",
      Key::F10 => "f10",
      Key::F11 => "f11",
      Key::F12 => "f12",
      Key::F13 => "f13",
      Key::F14 => "f14",
      Key::F15 => "f15",
      Key::F16 => "f16",
      Key::F17 => "f17",
      Key::F18 => "f18",
      Key::F19 => "f19",
      Key::F20 => "f20",
      Key::F21 => "f21",
      Key::F22 => "f22",
      Key::F23 => "f23",
      Key::F24 => "f24",

      Key::Cmd => "cmd",
      Key::Ctrl => "ctrl",
      Key::Alt => "alt",
      Key::Shift => "shift",
      Key::LCmd => "lcmd",
      Key::RCmd => "rcmd",
      Key::LCtrl => "lctrl",
      Key::RCtrl => "rctrl",
      Key::LAlt => "lalt",
      Key::RAlt => "ralt",
      Key::LShift => "lshift",
      Key::RShift => "rshift",
      Key::LWin => "lwin",
      Key::RWin => "rwin",
      Key::Win => "win",

      Key::Space => "space",
      Key::Tab => "tab",
      Key::Enter => "enter",
      Key::Delete => "delete",
      Key::Escape => "escape",
      Key::Backspace => "backspace",

      Key::Left => "left",
      Key::Right => "right",
      Key::Up => "up",
      Key::Down => "down",

      Key::Home => "home",
      Key::End => "end",
      Key::PageUp => "page_up",
      Key::PageDown => "page_down",
      Key::Insert => "insert",

      Key::Numpad0 => "numpad0",
      Key::Numpad1 => "numpad1",
      Key::Numpad2 => "numpad2",
      Key::Numpad3 => "numpad3",
      Key::Numpad4 => "numpad4",
      Key::Numpad5 => "numpad5",
      Key::Numpad6 => "numpad6",
      Key::Numpad7 => "numpad7",
      Key::Numpad8 => "numpad8",
      Key::Numpad9 => "numpad9",
      Key::NumpadAdd => "numpad_add",
      Key::NumpadSubtract => "numpad_subtract",
      Key::NumpadMultiply => "numpad_multiply",
      Key::NumpadDivide => "NumpadDivide",
      Key::NumpadDecimal => "numpad_decimal",
      Key::NumLock => "NumLock",
      Key::ScrollLock => "ScrollLock",
      Key::CapsLock => "CapsLock",
      Key::VolumeUp => "VolumeUp",
      Key::VolumeDown => "VolumeDown",
      Key::VolumeMute => "VolumeMute",
      Key::MediaNextTrack => "MediaNextTrack",
      Key::MediaPrevTrack => "MediaPrevTrack",
      Key::MediaStop => "MediaStop",
      Key::MediaPlayPause => "MediaPlayPause",
      Key::PrintScreen => "PrintScreen",
      Key::OemSemicolon => "oem_semicolon",
      Key::OemQuestion => "OemQuestion",
      Key::OemTilde => "OemTilde",
      Key::OemOpenBrackets => "oem_open_brackets",
      Key::OemPipe => "OemPipe",
      Key::OemCloseBrackets => "OemCloseBrackets",
      Key::OemQuotes => "oem_quotes",
      Key::OemPlus => "OemPlus",
      Key::OemComma => "OemComma",
      Key::OemMinus => "OemMinus",
      Key::OemPeriod => "oem_period",
      Key::Raw(keycode) => return write!(f, "KeyCode({keycode})"),
    };

    write!(f, "{s}")
  }
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

    // Should fail for unknown keys.
    assert!("invalid".parse::<Key>().is_err());
  }

  #[test]
  fn test_key_display() {
    assert_eq!(Key::A.to_string(), "a");
    assert_eq!(Key::Cmd.to_string(), "cmd");
    assert_eq!(Key::F1.to_string(), "f1");
    assert_eq!(Key::Space.to_string(), "space");
  }
}
