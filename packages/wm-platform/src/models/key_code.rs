#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
  VIRTUAL_KEY, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9,
  VK_A, VK_ADD, VK_B, VK_BACK, VK_C, VK_CAPITAL, VK_D, VK_DECIMAL,
  VK_DELETE, VK_DIVIDE, VK_DOWN, VK_E, VK_END, VK_ESCAPE, VK_F, VK_F1,
  VK_F10, VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16, VK_F17, VK_F18,
  VK_F19, VK_F2, VK_F20, VK_F21, VK_F22, VK_F23, VK_F24, VK_F3, VK_F4,
  VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_G, VK_H, VK_HOME, VK_I, VK_INSERT,
  VK_J, VK_K, VK_L, VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN,
  VK_M, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK,
  VK_MEDIA_STOP, VK_MULTIPLY, VK_N, VK_NEXT, VK_NUMLOCK, VK_NUMPAD0,
  VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6,
  VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9, VK_O, VK_OEM_1, VK_OEM_2, VK_OEM_3,
  VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7, VK_OEM_COMMA, VK_OEM_MINUS,
  VK_OEM_PERIOD, VK_OEM_PLUS, VK_P, VK_PRIOR, VK_Q, VK_R, VK_RCONTROL,
  VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_S, VK_SCROLL,
  VK_SNAPSHOT, VK_SPACE, VK_SUBTRACT, VK_T, VK_TAB, VK_U, VK_UP, VK_V,
  VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP, VK_W, VK_X, VK_Y, VK_Z,
};

use crate::{Key, KeyCode};

#[derive(Debug, thiserror::Error)]
pub enum KeyConversionError {
  #[error("Unknown key code: {0}")]
  UnknownKeyCode(KeyCode),
}

/// Generates `From` and `TryFrom` implementations for converting between
/// `Key` and `KeyCode`.
///
/// For Windows, the key code is assumed to be a `VK_*` constant (accessed
/// via .0).
///
/// # Example
/// ```no_run,compile_fail
/// impl_key_code_conversion! {
///   Enter => { windows: VK_RETURN, macos: 0x24, },
///   Space => { windows: VK_SPACE, macos: 0x31, },
///   PrintScreen => { windows: VK_SNAPSHOT, }, // Only supported on Windows.
/// }
/// ```
macro_rules! impl_key_code_conversion {
  (
    $(
      $variant:ident => {
        $(windows: $win_code:expr,)?
        $(macos: $mac_code:expr,)?
      }
    ),* $(,)?
  ) => {
    #[cfg(target_os = "windows")]
    impl TryFrom<KeyCode> for Key {
      type Error = KeyConversionError;

      fn try_from(key_code: KeyCode) -> Result<Self, Self::Error> {
        // LINT: Allow unreachable patterns since modifier keys are
        // duplicated (e.g. `LShift` and `Shift`).
        match VIRTUAL_KEY(key_code.0) {
          $($($win_code => Ok(Key::$variant),)?)*
          _ => Err(KeyConversionError::UnknownKeyCode(key_code)),
        }
      }
    }

    #[cfg(target_os = "macos")]
    impl TryFrom<KeyCode> for Key {
      type Error = KeyConversionError;

      fn try_from(key_code: KeyCode) -> Result<Self, Self::Error> {
        // LINT: Allow unreachable patterns since modifier keys are
        // duplicated (e.g. `LShift` and `Shift`).
        #[allow(unreachable_patterns)]
        match key_code.0 {
          $($($mac_code => Ok(Key::$variant),)?)*
          _ => Err(KeyConversionError::UnknownKeyCode(key_code)),
        }
      }
    }

    impl TryFrom<Key> for KeyCode {
      type Error = KeyConversionError;

      fn try_from(key: Key) -> Result<Self, Self::Error> {
        match key {
          $(Key::$variant => {
            #[cfg(target_os = "windows")]
            {
              $(return Ok(KeyCode($win_code.0));)?
              #[allow(unreachable_code)]
              return Err(KeyConversionError::UnknownKeyCode(KeyCode(0)));
            }
            #[cfg(target_os = "macos")]
            {
              $(return Ok(KeyCode($mac_code));)?
              #[allow(unreachable_code)]
              return Err(KeyConversionError::UnknownKeyCode(KeyCode(0)));
            }
          }),*
        }
      }
    }
  };
}

impl_key_code_conversion! {
  // Letter keys
  A => { windows: VK_A, macos: 0x00, },
  B => { windows: VK_B, macos: 0x0B, },
  C => { windows: VK_C, macos: 0x08, },
  D => { windows: VK_D, macos: 0x02, },
  E => { windows: VK_E, macos: 0x0E, },
  F => { windows: VK_F, macos: 0x03, },
  G => { windows: VK_G, macos: 0x05, },
  H => { windows: VK_H, macos: 0x04, },
  I => { windows: VK_I, macos: 0x22, },
  J => { windows: VK_J, macos: 0x26, },
  K => { windows: VK_K, macos: 0x28, },
  L => { windows: VK_L, macos: 0x25, },
  M => { windows: VK_M, macos: 0x2E, },
  N => { windows: VK_N, macos: 0x2D, },
  O => { windows: VK_O, macos: 0x1F, },
  P => { windows: VK_P, macos: 0x23, },
  Q => { windows: VK_Q, macos: 0x0C, },
  R => { windows: VK_R, macos: 0x0F, },
  S => { windows: VK_S, macos: 0x01, },
  T => { windows: VK_T, macos: 0x11, },
  U => { windows: VK_U, macos: 0x20, },
  V => { windows: VK_V, macos: 0x09, },
  W => { windows: VK_W, macos: 0x0D, },
  X => { windows: VK_X, macos: 0x07, },
  Y => { windows: VK_Y, macos: 0x10, },
  Z => { windows: VK_Z, macos: 0x06, },
  // Number keys
  D0 => { windows: VK_0, macos: 0x1D, },
  D1 => { windows: VK_1, macos: 0x12, },
  D2 => { windows: VK_2, macos: 0x13, },
  D3 => { windows: VK_3, macos: 0x14, },
  D4 => { windows: VK_4, macos: 0x15, },
  D5 => { windows: VK_5, macos: 0x17, },
  D6 => { windows: VK_6, macos: 0x16, },
  D7 => { windows: VK_7, macos: 0x1A, },
  D8 => { windows: VK_8, macos: 0x1C, },
  D9 => { windows: VK_9, macos: 0x19, },
  // Function keys
  F1 => { windows: VK_F1, macos: 0x7A, },
  F2 => { windows: VK_F2, macos: 0x78, },
  F3 => { windows: VK_F3, macos: 0x63, },
  F4 => { windows: VK_F4, macos: 0x76, },
  F5 => { windows: VK_F5, macos: 0x60, },
  F6 => { windows: VK_F6, macos: 0x61, },
  F7 => { windows: VK_F7, macos: 0x62, },
  F8 => { windows: VK_F8, macos: 0x64, },
  F9 => { windows: VK_F9, macos: 0x65, },
  F10 => { windows: VK_F10, macos: 0x6D, },
  F11 => { windows: VK_F11, macos: 0x67, },
  F12 => { windows: VK_F12, macos: 0x6F, },
  F13 => { windows: VK_F13, macos: 0x69, },
  F14 => { windows: VK_F14, macos: 0x6B, },
  F15 => { windows: VK_F15, macos: 0x71, },
  F16 => { windows: VK_F16, macos: 0x6A, },
  F17 => { windows: VK_F17, macos: 0x40, },
  F18 => { windows: VK_F18, macos: 0x4F, },
  F19 => { windows: VK_F19, macos: 0x50, },
  F20 => { windows: VK_F20, macos: 0x5A, },
  // Windows-only function keys; macOS has no F21-F24.
  F21 => { windows: VK_F21, },
  F22 => { windows: VK_F22, },
  F23 => { windows: VK_F23, },
  F24 => { windows: VK_F24, },
  // Modifier keys - use platform-specific primary variants
  LShift => { windows: VK_LSHIFT, macos: 0x38, },
  RShift => { windows: VK_RSHIFT, macos: 0x3C, },
  LCtrl => { windows: VK_LCONTROL, macos: 0x3B, },
  RCtrl => { windows: VK_RCONTROL, macos: 0x3E, },
  LAlt => { windows: VK_LMENU, macos: 0x3A, },
  RAlt => { windows: VK_RMENU, macos: 0x3D, },
  // General modifiers (canonical mapping)
  Shift => { windows: VK_LSHIFT, macos: 0x38, },
  Ctrl => { windows: VK_LCONTROL, macos: 0x3B, },
  Alt => { windows: VK_LMENU, macos: 0x3A, },
  Cmd => { windows: VK_LWIN, macos: 0x37, },
  Win => { windows: VK_LWIN, macos: 0x37, },
  // Platform-specific key mappings (aliases)
  LWin => { windows: VK_LWIN, macos: 0x37, },
  RWin => { windows: VK_RWIN, macos: 0x36, },
  LCmd => { windows: VK_LWIN, macos: 0x37, },
  RCmd => { windows: VK_RWIN, macos: 0x36, },
  // Special keys
  Space => { windows: VK_SPACE, macos: 0x31, },
  Tab => { windows: VK_TAB, macos: 0x30, },
  Enter => { windows: VK_RETURN, macos: 0x24, },
  // macOS: Backspace == 0x33, Forward Delete == 0x75
  Delete => { windows: VK_DELETE, macos: 0x75, },
  Escape => { windows: VK_ESCAPE, macos: 0x35, },
  Backspace => { windows: VK_BACK, macos: 0x33, },
  // Arrow keys
  Left => { windows: VK_LEFT, macos: 0x7B, },
  Right => { windows: VK_RIGHT, macos: 0x7C, },
  Up => { windows: VK_UP, macos: 0x7E, },
  Down => { windows: VK_DOWN, macos: 0x7D, },
  // Navigation keys
  Home => { windows: VK_HOME, macos: 0x73, },
  End => { windows: VK_END, macos: 0x77, },
  PageUp => { windows: VK_PRIOR, macos: 0x74, },
  PageDown => { windows: VK_NEXT, macos: 0x79, },
  Insert => { windows: VK_INSERT, macos: 0x72, }, // Note: macOS 0x72 is Help
  // OEM keys
  OemSemicolon => { windows: VK_OEM_1, macos: 0x29, },
  OemQuestion => { windows: VK_OEM_2, macos: 0x2C, },
  OemTilde => { windows: VK_OEM_3, macos: 0x32, },
  OemOpenBrackets => { windows: VK_OEM_4, macos: 0x21, },
  OemPipe => { windows: VK_OEM_5, macos: 0x2A, },
  OemCloseBrackets => { windows: VK_OEM_6, macos: 0x1E, },
  OemQuotes => { windows: VK_OEM_7, macos: 0x27, },
  OemPlus => { windows: VK_OEM_PLUS, macos: 0x18, },
  OemComma => { windows: VK_OEM_COMMA, macos: 0x2B, },
  OemMinus => { windows: VK_OEM_MINUS, macos: 0x1B, },
  OemPeriod => { windows: VK_OEM_PERIOD, macos: 0x2F, },
  // Numpad
  Numpad0 => { windows: VK_NUMPAD0, macos: 0x52, },
  Numpad1 => { windows: VK_NUMPAD1, macos: 0x53, },
  Numpad2 => { windows: VK_NUMPAD2, macos: 0x54, },
  Numpad3 => { windows: VK_NUMPAD3, macos: 0x55, },
  Numpad4 => { windows: VK_NUMPAD4, macos: 0x56, },
  Numpad5 => { windows: VK_NUMPAD5, macos: 0x57, },
  Numpad6 => { windows: VK_NUMPAD6, macos: 0x58, },
  Numpad7 => { windows: VK_NUMPAD7, macos: 0x59, },
  Numpad8 => { windows: VK_NUMPAD8, macos: 0x5B, },
  Numpad9 => { windows: VK_NUMPAD9, macos: 0x5C, },
  NumpadAdd => { windows: VK_ADD, macos: 0x45, },
  NumpadSubtract => { windows: VK_SUBTRACT, macos: 0x4E, },
  NumpadMultiply => { windows: VK_MULTIPLY, macos: 0x43, },
  NumpadDivide => { windows: VK_DIVIDE, macos: 0x4B, },
  NumpadDecimal => { windows: VK_DECIMAL, macos: 0x41, },
  // Lock keys
  NumLock => { windows: VK_NUMLOCK, macos: 0x47, },
  ScrollLock => { windows: VK_SCROLL, macos: 0x6B, },
  CapsLock => { windows: VK_CAPITAL, macos: 0x39, },
  // Media keys
  VolumeUp => { windows: VK_VOLUME_UP, macos: 0x48, },
  VolumeDown => { windows: VK_VOLUME_DOWN, macos: 0x49, },
  VolumeMute => { windows: VK_VOLUME_MUTE, macos: 0x4A, },
  // TODO: Verify these media keys for macOS.
  MediaNextTrack => { windows: VK_MEDIA_NEXT_TRACK, macos: 0x42, },
  MediaPrevTrack => { windows: VK_MEDIA_PREV_TRACK, macos: 0x4D, },
  MediaStop => { windows: VK_MEDIA_STOP, macos: 0x4C, },
  MediaPlayPause => { windows: VK_MEDIA_PLAY_PAUSE, macos: 0x34, },
  // Print screen
  PrintScreen => { windows: VK_SNAPSHOT, },
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_key_conversion_roundtrip() {
    let test_keys = [
      Key::A,
      Key::S,
      Key::D,
      Key::F,
      Key::Cmd,
      Key::LAlt,
      Key::RCtrl,
      Key::LShift,
      Key::Space,
      Key::Tab,
      Key::Enter,
      Key::F1,
      Key::F12,
      Key::Left,
      Key::Right,
    ];

    for key in test_keys {
      let code: KeyCode = key.try_into().unwrap();
      let key2: Key = code.try_into().unwrap();
      assert_eq!(key, key2, "Roundtrip failed for key: {key:?}");
    }
  }
}
