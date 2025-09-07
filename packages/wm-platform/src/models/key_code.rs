#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
  VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9, VK_A,
  VK_ADD, VK_B, VK_BACK, VK_C, VK_CAPITAL, VK_D, VK_DECIMAL, VK_DELETE,
  VK_DIVIDE, VK_DOWN, VK_E, VK_END, VK_ESCAPE, VK_F, VK_F1, VK_F10,
  VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16, VK_F17, VK_F18, VK_F19,
  VK_F2, VK_F20, VK_F21, VK_F22, VK_F23, VK_F24, VK_F3, VK_F4, VK_F5,
  VK_F6, VK_F7, VK_F8, VK_F9, VK_G, VK_H, VK_HOME, VK_I, VK_INSERT, VK_J,
  VK_K, VK_L, VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_M,
  VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK,
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

/// Macro to generate trait implementations for `Key` <-> `KeyCode`
/// conversion.
///
/// Each input line specifies:
/// - The `Key` enum variant
/// - The Windows key code (u16)
/// - The macOS key code (i64)
macro_rules! create_key_code_mapping {
  ($(($key:ident, $win_code:expr, $mac_code:expr)),* $(,)?) => {
    #[cfg(target_os = "windows")]
    impl From<KeyCode> for Key {
      fn from(keycode: KeyCode) -> Self {
        match VIRTUAL_KEY(keycode) {
          $(
            $win_code => Key::$key,
          )*
          _ => Key::Raw(keycode),
        }
      }
    }

    #[cfg(target_os = "macos")]
    impl From<KeyCode> for Key {
      fn from(keycode: KeyCode) -> Self {
        match keycode.0 {
          $(
            $mac_code => Key::$key,
          )*
          _ => Key::Raw(keycode),
        }
      }
    }

    impl From<Key> for KeyCode {
      fn from(key: Key) -> Self {
        match key {
          $(Key::$key => {
            #[cfg(target_os = "windows")]
            { KeyCode($win_code) }
            #[cfg(target_os = "macos")]
            { KeyCode($mac_code) }
          })*
          Key::Raw(keycode) => keycode,
        }
      }
    }
  };
}

// Define all key code mappings.
//
// Each line is a tuple of the form:
// (`Key` enum variant, Windows key code, macOS key code).
create_key_code_mapping![
  // Letter keys
  (A, VK_A, 0x00),
  (B, VK_B, 0x0B),
  (C, VK_C, 0x08),
  (D, VK_D, 0x02),
  (E, VK_E, 0x0E),
  (F, VK_F, 0x03),
  (G, VK_G, 0x05),
  (H, VK_H, 0x04),
  (I, VK_I, 0x22),
  (J, VK_J, 0x26),
  (K, VK_K, 0x28),
  (L, VK_L, 0x25),
  (M, VK_M, 0x2E),
  (N, VK_N, 0x2D),
  (O, VK_O, 0x1F),
  (P, VK_P, 0x23),
  (Q, VK_Q, 0x0C),
  (R, VK_R, 0x0F),
  (S, VK_S, 0x01),
  (T, VK_T, 0x11),
  (U, VK_U, 0x20),
  (V, VK_V, 0x09),
  (W, VK_W, 0x0D),
  (X, VK_X, 0x07),
  (Y, VK_Y, 0x10),
  (Z, VK_Z, 0x06),
  // Number keys
  (D0, VK_0, 0x1D),
  (D1, VK_1, 0x12),
  (D2, VK_2, 0x13),
  (D3, VK_3, 0x14),
  (D4, VK_4, 0x15),
  (D5, VK_5, 0x17),
  (D6, VK_6, 0x16),
  (D7, VK_7, 0x1A),
  (D8, VK_8, 0x1C),
  (D9, VK_9, 0x19),
  // Function keys
  (F1, VK_F1, 0x7A),
  (F2, VK_F2, 0x78),
  (F3, VK_F3, 0x63),
  (F4, VK_F4, 0x76),
  (F5, VK_F5, 0x60),
  (F6, VK_F6, 0x61),
  (F7, VK_F7, 0x62),
  (F8, VK_F8, 0x64),
  (F9, VK_F9, 0x65),
  (F10, VK_F10, 0x6D),
  (F11, VK_F11, 0x67),
  (F12, VK_F12, 0x6F),
  (F13, VK_F13, 0x69),
  (F14, VK_F14, 0x6B),
  (F15, VK_F15, 0x71),
  (F16, VK_F16, 0x6A),
  (F17, VK_F17, 0x40),
  (F18, VK_F18, 0x4F),
  (F19, VK_F19, 0x50),
  (F20, VK_F20, 0x5A),
  // Windows-only function keys; macOS has no F21-F24.
  (F21, VK_F21, -1),
  (F22, VK_F22, -1),
  (F23, VK_F23, -1),
  (F24, VK_F24, -1),
  // Modifier keys - use platform-specific primary variants
  (LShift, VK_LSHIFT, 0x38),
  (RShift, VK_RSHIFT, 0x3C),
  (LCtrl, VK_LCONTROL, 0x3B),
  (RCtrl, VK_RCONTROL, 0x3E),
  (LAlt, VK_LMENU, 0x3A),
  (RAlt, VK_RMENU, 0x3D),
  // General modifiers (canonical mapping)
  (Shift, VK_LSHIFT, 0x38),
  (Ctrl, VK_LCONTROL, 0x3B),
  (Alt, VK_LMENU, 0x3A),
  (Cmd, VK_LWIN, 0x37),
  (Win, VK_LWIN, 0x37),
  // Platform-specific key mappings (aliases)
  (LWin, VK_LWIN, 0x37),
  (RWin, VK_RWIN, 0x36),
  (LCmd, VK_LWIN, 0x37),
  (RCmd, VK_RWIN, 0x36),
  // Special keys
  (Space, VK_SPACE, 0x31),
  (Tab, VK_TAB, 0x30),
  (Enter, VK_RETURN, 0x24),
  // macOS: Backspace == 0x33, Forward Delete == 0x75
  (Delete, VK_DELETE, 0x75),
  (Escape, VK_ESCAPE, 0x35),
  (Backspace, VK_BACK, 0x33),
  // Arrow keys
  (Left, VK_LEFT, 0x7B),
  (Right, VK_RIGHT, 0x7C),
  (Up, VK_UP, 0x7E),
  (Down, VK_DOWN, 0x7D),
  // Navigation keys
  (Home, VK_HOME, 0x73),
  (End, VK_END, 0x77),
  (PageUp, VK_PRIOR, 0x74),
  (PageDown, VK_NEXT, 0x79),
  (Insert, VK_INSERT, 0x72), // Note: macOS 0x72 is Help
  // OEM explicit variants (Windows)
  (OemSemicolon, VK_OEM_1, 0x29),
  (OemQuestion, VK_OEM_2, 0x2C),
  (OemTilde, VK_OEM_3, 0x32),
  (OemOpenBrackets, VK_OEM_4, 0x21),
  (OemPipe, VK_OEM_5, 0x2A),
  (OemCloseBrackets, VK_OEM_6, 0x1E),
  (OemQuotes, VK_OEM_7, 0x27),
  (OemPlus, VK_OEM_PLUS, 0x18),
  (OemComma, VK_OEM_COMMA, 0x2B),
  (OemMinus, VK_OEM_MINUS, 0x1B),
  (OemPeriod, VK_OEM_PERIOD, 0x2F),
  // Numpad
  (Numpad0, VK_NUMPAD0, 0x52),
  (Numpad1, VK_NUMPAD1, 0x53),
  (Numpad2, VK_NUMPAD2, 0x54),
  (Numpad3, VK_NUMPAD3, 0x55),
  (Numpad4, VK_NUMPAD4, 0x56),
  (Numpad5, VK_NUMPAD5, 0x57),
  (Numpad6, VK_NUMPAD6, 0x58),
  (Numpad7, VK_NUMPAD7, 0x59),
  (Numpad8, VK_NUMPAD8, 0x5B),
  (Numpad9, VK_NUMPAD9, 0x5C),
  (NumpadAdd, VK_ADD, 0x45),
  (NumpadSubtract, VK_SUBTRACT, 0x4E),
  (NumpadMultiply, VK_MULTIPLY, 0x43),
  (NumpadDivide, VK_DIVIDE, 0x4B),
  (NumpadDecimal, VK_DECIMAL, 0x41),
  // Lock keys
  (NumLock, VK_NUMLOCK, 0x47),
  (ScrollLock, VK_SCROLL, 0x6B),
  (CapsLock, VK_CAPITAL, 0x39),
  // Media keys (Windows codes are standard; macOS values are
  // placeholders)
  (VolumeUp, VK_VOLUME_UP, 0x48),
  (VolumeDown, VK_VOLUME_DOWN, 0x49),
  (VolumeMute, VK_VOLUME_MUTE, 0x4A),
  (MediaNextTrack, VK_MEDIA_NEXT_TRACK, 0x42),
  (MediaPrevTrack, VK_MEDIA_PREV_TRACK, 0x4D),
  (MediaStop, VK_MEDIA_STOP, 0x4C),
  (MediaPlayPause, VK_MEDIA_PLAY_PAUSE, 0x34),
  // Print screen
  (PrintScreen, VK_SNAPSHOT, -1),
];

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_key_conversion_roundtrip() {
    // Test `KeyCode -> `Key` -> `KeyCode` conversion.
    for raw_key_code in 0..254 {
      let key_code = KeyCode(raw_key_code);
      let key: Key = key_code.into();
      let converted_back: KeyCode = key.into();

      assert_eq!(
        key_code, converted_back,
        "Roundtrip failed for key: {key:?}"
      );
    }
  }
}
