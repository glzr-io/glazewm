use windows::Win32::UI::Input::KeyboardAndMouse::{
  VIRTUAL_KEY, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9,
  VK_A, VK_ADD, VK_B, VK_BACK, VK_C, VK_CAPITAL, VK_CONVERT, VK_D,
  VK_DECIMAL, VK_DELETE, VK_DIVIDE, VK_DOWN, VK_E, VK_END, VK_ESCAPE,
  VK_F, VK_F1, VK_F10, VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16,
  VK_F17, VK_F18, VK_F19, VK_F2, VK_F20, VK_F21, VK_F22, VK_F23, VK_F24,
  VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_G, VK_H, VK_HOME,
  VK_I, VK_INSERT, VK_J, VK_K, VK_L, VK_LCONTROL, VK_LEFT, VK_LMENU,
  VK_LSHIFT, VK_LWIN, VK_M, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE,
  VK_MEDIA_PREV_TRACK, VK_MEDIA_STOP, VK_MULTIPLY, VK_N, VK_NEXT,
  VK_NONCONVERT, VK_NUMLOCK, VK_NUMPAD0, VK_NUMPAD1, VK_NUMPAD2,
  VK_NUMPAD3, VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8,
  VK_NUMPAD9, VK_O, VK_OEM_1, VK_OEM_102, VK_OEM_2, VK_OEM_3, VK_OEM_4,
  VK_OEM_5, VK_OEM_6, VK_OEM_7, VK_OEM_8, VK_OEM_COMMA, VK_OEM_MINUS,
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

/// Generates `TryFrom` implementations for converting between `Key` and
/// `KeyCode`.
///
/// For Windows, the key code is assumed to be a `VK_*` constant (accessed
/// via .0).
///
/// # Example
/// ```no_run,compile_fail
/// impl_key_code_conversion! {
///   Enter => { windows: VK_RETURN, },
///   Space => { windows: VK_SPACE, },
///   PrintScreen => { windows: VK_SNAPSHOT, }, // Only supported on Windows.
/// }
/// ```
macro_rules! impl_key_code_conversion {
  (
    $(
      $variant:ident => {
        $(windows: $win_code:expr,)?
      }
    ),* $(,)?
  ) => {
    impl TryFrom<KeyCode> for Key {
      type Error = KeyConversionError;

      fn try_from(key_code: KeyCode) -> Result<Self, Self::Error> {
        let vk = VIRTUAL_KEY(key_code.0);
        $($(if vk == $win_code { return Ok(Key::$variant); })?)*
        Err(KeyConversionError::UnknownKeyCode(key_code))
      }
    }


    impl TryFrom<Key> for KeyCode {
      type Error = KeyConversionError;

      fn try_from(key: Key) -> Result<Self, Self::Error> {
        match key {
          $(Key::$variant => {
            $(return Ok(KeyCode($win_code.0));)?
            #[allow(unreachable_code)]
            return Err(KeyConversionError::UnknownKeyCode(KeyCode(0)));
          }),*
        }
      }
    }
  };
}

impl_key_code_conversion! {
  // Letter keys
  A => { windows: VK_A, },
  B => { windows: VK_B, },
  C => { windows: VK_C, },
  D => { windows: VK_D, },
  E => { windows: VK_E, },
  F => { windows: VK_F, },
  G => { windows: VK_G, },
  H => { windows: VK_H, },
  I => { windows: VK_I, },
  J => { windows: VK_J, },
  K => { windows: VK_K, },
  L => { windows: VK_L, },
  M => { windows: VK_M, },
  N => { windows: VK_N, },
  O => { windows: VK_O, },
  P => { windows: VK_P, },
  Q => { windows: VK_Q, },
  R => { windows: VK_R, },
  S => { windows: VK_S, },
  T => { windows: VK_T, },
  U => { windows: VK_U, },
  V => { windows: VK_V, },
  W => { windows: VK_W, },
  X => { windows: VK_X, },
  Y => { windows: VK_Y, },
  Z => { windows: VK_Z, },
  // Number keys
  D0 => { windows: VK_0, },
  D1 => { windows: VK_1, },
  D2 => { windows: VK_2, },
  D3 => { windows: VK_3, },
  D4 => { windows: VK_4, },
  D5 => { windows: VK_5, },
  D6 => { windows: VK_6, },
  D7 => { windows: VK_7, },
  D8 => { windows: VK_8, },
  D9 => { windows: VK_9, },
  // Function keys
  F1 => { windows: VK_F1, },
  F2 => { windows: VK_F2, },
  F3 => { windows: VK_F3, },
  F4 => { windows: VK_F4, },
  F5 => { windows: VK_F5, },
  F6 => { windows: VK_F6, },
  F7 => { windows: VK_F7, },
  F8 => { windows: VK_F8, },
  F9 => { windows: VK_F9, },
  F10 => { windows: VK_F10, },
  F11 => { windows: VK_F11, },
  F12 => { windows: VK_F12, },
  F13 => { windows: VK_F13, },
  F14 => { windows: VK_F14, },
  F15 => { windows: VK_F15, },
  F16 => { windows: VK_F16, },
  F17 => { windows: VK_F17, },
  F18 => { windows: VK_F18, },
  F19 => { windows: VK_F19, },
  F20 => { windows: VK_F20, },
  F21 => { windows: VK_F21, },
  F22 => { windows: VK_F22, },
  F23 => { windows: VK_F23, },
  F24 => { windows: VK_F24, },
  // Modifier keys - use platform-specific primary variants
  LShift => { windows: VK_LSHIFT, },
  RShift => { windows: VK_RSHIFT, },
  LCtrl => { windows: VK_LCONTROL, },
  RCtrl => { windows: VK_RCONTROL, },
  LAlt => { windows: VK_LMENU, },
  RAlt => { windows: VK_RMENU, },
  // General modifiers (canonical mapping)
  Shift => { windows: VK_LSHIFT, },
  Ctrl => { windows: VK_LCONTROL, },
  Alt => { windows: VK_LMENU, },
  Cmd => { },
  Win => { windows: VK_LWIN, },
  // Platform-specific key mappings (aliases)
  LWin => { windows: VK_LWIN, },
  RWin => { windows: VK_RWIN, },
  LCmd => { },
  RCmd => { },
  // Special keys
  Space => { windows: VK_SPACE, },
  Tab => { windows: VK_TAB, },
  Enter => { windows: VK_RETURN, },
  Delete => { windows: VK_DELETE, },
  Escape => { windows: VK_ESCAPE, },
  Backspace => { windows: VK_BACK, },
  // Arrow keys
  Left => { windows: VK_LEFT, },
  Right => { windows: VK_RIGHT, },
  Up => { windows: VK_UP, },
  Down => { windows: VK_DOWN, },
  // Navigation keys
  Home => { windows: VK_HOME, },
  End => { windows: VK_END, },
  PageUp => { windows: VK_PRIOR, },
  PageDown => { windows: VK_NEXT, },
  Insert => { windows: VK_INSERT, },
  // OEM keys
  OemSemicolon => { windows: VK_OEM_1, },
  OemQuestion => { windows: VK_OEM_2, },
  OemTilde => { windows: VK_OEM_3, },
  OemOpenBrackets => { windows: VK_OEM_4, },
  OemPipe => { windows: VK_OEM_5, },
  OemCloseBrackets => { windows: VK_OEM_6, },
  OemQuotes => { windows: VK_OEM_7, },
  Oem8 => { windows: VK_OEM_8, },
  Oem102 => { windows: VK_OEM_102, },
  OemPlus => { windows: VK_OEM_PLUS, },
  OemComma => { windows: VK_OEM_COMMA, },
  OemMinus => { windows: VK_OEM_MINUS, },
  OemPeriod => { windows: VK_OEM_PERIOD, },
  // Numpad
  Numpad0 => { windows: VK_NUMPAD0, },
  Numpad1 => { windows: VK_NUMPAD1, },
  Numpad2 => { windows: VK_NUMPAD2, },
  Numpad3 => { windows: VK_NUMPAD3, },
  Numpad4 => { windows: VK_NUMPAD4, },
  Numpad5 => { windows: VK_NUMPAD5, },
  Numpad6 => { windows: VK_NUMPAD6, },
  Numpad7 => { windows: VK_NUMPAD7, },
  Numpad8 => { windows: VK_NUMPAD8, },
  Numpad9 => { windows: VK_NUMPAD9, },
  NumpadAdd => { windows: VK_ADD, },
  NumpadSubtract => { windows: VK_SUBTRACT, },
  NumpadMultiply => { windows: VK_MULTIPLY, },
  NumpadDivide => { windows: VK_DIVIDE, },
  NumpadDecimal => { windows: VK_DECIMAL, },
  // Lock keys
  NumLock => { windows: VK_NUMLOCK, },
  ScrollLock => { windows: VK_SCROLL, },
  CapsLock => { windows: VK_CAPITAL, },
  // Media keys
  VolumeUp => { windows: VK_VOLUME_UP, },
  VolumeDown => { windows: VK_VOLUME_DOWN, },
  VolumeMute => { windows: VK_VOLUME_MUTE, },
  MediaNextTrack => { windows: VK_MEDIA_NEXT_TRACK, },
  MediaPrevTrack => { windows: VK_MEDIA_PREV_TRACK, },
  MediaStop => { windows: VK_MEDIA_STOP, },
  MediaPlayPause => { windows: VK_MEDIA_PLAY_PAUSE, },
  PrintScreen => { windows: VK_SNAPSHOT, },
  // Language-specific keys
  Muhenkan => { windows: VK_NONCONVERT, },
  Henkan => { windows: VK_CONVERT, },
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

  #[test]
  fn test_platform_specific_key_code() {
    let code = KeyCode::try_from(Key::Win);
    assert!(code.is_ok());
    let code2 = KeyCode::try_from(Key::Cmd);
    assert!(code2.is_err());
  }
}
