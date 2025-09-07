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
  Win,
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

  // Navigation keys
  Home,
  End,
  PageUp,
  PageDown,
  Insert,

  // Lock keys
  NumLock,
  ScrollLock,
  CapsLock,

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
}

impl Key {
  /// Attempts to parse a key from a literal string (e.g. `a`, `;`, `à`).
  ///
  /// # Platform-specific
  ///
  /// - **macOS**: Not implemented. Returns `KeyParseError::UnknownKey` for
  ///   all keys.
  ///
  /// # Errors
  ///
  /// Returns `KeyParseError::UnknownKey` if the key is not found on the
  /// current keyboard layout.
  pub fn try_from_literal(key_str: &str) -> Result<Self, KeyParseError> {
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

/// Generates `FromStr` and `Display` implementations for the `Key` enum.
///
/// Each variant can have multiple string aliases, with the first used for
/// the `Display` implementation.
///
/// # Example
/// ```
/// impl_key_parsing! {
///   Enter => ["enter", "return", "cr"],
///   Space => ["space", "spacebar", " "],
/// }
/// ```
macro_rules! impl_key_parsing {
  ($( $variant:ident => [$($str_name:literal),+ $(,)?]),* $(,)?) => {
    impl FromStr for Key {
      type Err = KeyParseError;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
          $($($str_name)|+ => Ok(Key::$variant),)*
          _ => Err(KeyParseError::UnknownKey(s.to_string())),
        }
      }
    }

    impl fmt::Display for Key {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
          $(Key::$variant => {
            // Return the first string alias as the display name.
            let aliases = &[$($str_name),+];
            write!(f, "{}", aliases[0])
          },)*
          _ => write!(f, "Unknown"),
        }
      }
    }

    impl Key {
      /// Returns all string aliases for this key variant.
      pub fn all_aliases(&self) -> Option<&'static [&'static str]> {
        match self {
          $(Key::$variant => Some(&[$($str_name),+]),)*
          _ => None,
        }
      }
    }
  };
}

impl_key_parsing! {
  // Letter keys
  A => ["a"], B => ["b"], C => ["c"], D => ["d"], E => ["e"], F => ["f"],
  G => ["g"], H => ["h"], I => ["i"], J => ["j"], K => ["k"], L => ["l"],
  M => ["m"], N => ["n"], O => ["o"], P => ["p"], Q => ["q"], R => ["r"],
  S => ["s"], T => ["t"], U => ["u"], V => ["v"], W => ["w"], X => ["x"],
  Y => ["y"], Z => ["z"],

  // Number keys
  D0 => ["0", "d0"],
  D1 => ["1", "d1"],
  D2 => ["2", "d2"],
  D3 => ["3", "d3"],
  D4 => ["4", "d4"],
  D5 => ["5", "d5"],
  D6 => ["6", "d6"],
  D7 => ["7", "d7"],
  D8 => ["8", "d8"],
  D9 => ["9", "d9"],

  // Function keys
  F1 => ["f1"], F2 => ["f2"], F3 => ["f3"], F4 => ["f4"], F5 => ["f5"],
  F6 => ["f6"], F7 => ["f7"], F8 => ["f8"], F9 => ["f9"], F10 => ["f10"],
  F11 => ["f11"], F12 => ["f12"], F13 => ["f13"], F14 => ["f14"],
  F15 => ["f15"], F16 => ["f16"], F17 => ["f17"], F18 => ["f18"],
  F19 => ["f19"], F20 => ["f20"], F21 => ["f21"], F22 => ["f22"],
  F23 => ["f23"], F24 => ["f24"],

  // Modifier keys
  Cmd => ["cmd"],
  Ctrl => ["ctrl", "control"],
  Alt => ["alt", "menu"],
  Shift => ["shift"],
  Win => ["win"],
  LCmd => ["lcmd"],
  RCmd => ["rcmd"],
  LCtrl => ["lctrl"],
  RCtrl => ["rctrl"],
  LAlt => ["lalt", "lmenu"],
  RAlt => ["ralt", "rmenu"],
  LShift => ["lshift"],
  RShift => ["rshift"],
  LWin => ["lwin"],
  RWin => ["rwin"],

  // Special keys
  Space => ["space"],
  Tab => ["tab"],
  Enter => ["enter", "return"],
  Delete => ["delete"],
  Escape => ["escape"],
  Backspace => ["backspace"],

  // Arrow keys
  Left => ["left"],
  Right => ["right"],
  Up => ["up"],
  Down => ["down"],

  // Navigation keys
  Home => ["home"],
  End => ["end"],
  PageUp => ["page_up"],
  PageDown => ["page_down"],
  Insert => ["insert"],

  // Lock keys
  NumLock => ["num_lock"],
  ScrollLock => ["scroll_lock"],
  CapsLock => ["caps_lock"],

  // Numpad
  Numpad0 => ["numpad0"],
  Numpad1 => ["numpad1"],
  Numpad2 => ["numpad2"],
  Numpad3 => ["numpad3"],
  Numpad4 => ["numpad4"],
  Numpad5 => ["numpad5"],
  Numpad6 => ["numpad6"],
  Numpad7 => ["numpad7"],
  Numpad8 => ["numpad8"],
  Numpad9 => ["numpad9"],
  NumpadAdd => ["numpad_add", "add"],
  NumpadSubtract => ["numpad_subtract", "subtract"],
  NumpadMultiply => ["numpad_multiply", "multiply"],
  NumpadDivide => ["numpad_divide", "divide"],
  NumpadDecimal => ["numpad_decimal", "decimal"],

  // Media keys
  VolumeUp => ["volume_up"],
  VolumeDown => ["volume_down"],
  VolumeMute => ["volume_mute"],
  MediaNextTrack => ["media_next_track"],
  MediaPrevTrack => ["media_prev_track"],
  MediaStop => ["media_stop"],
  MediaPlayPause => ["media_play_pause"],
  PrintScreen => ["print_screen"],

  // OEM keys
  OemSemicolon => ["oem_semicolon"],
  OemQuestion => ["oem_question"],
  OemTilde => ["oem_tilde"],
  OemOpenBrackets => ["oem_open_brackets"],
  OemPipe => ["oem_pipe"],
  OemCloseBrackets => ["oem_close_brackets"],
  OemQuotes => ["oem_quotes"],
  OemPlus => ["oem_plus"],
  OemComma => ["oem_comma"],
  OemMinus => ["oem_minus"],
  OemPeriod => ["oem_period"],
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
