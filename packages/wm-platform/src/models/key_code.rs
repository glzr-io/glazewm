use crate::{Key, KeyCode};

/// Mapping of platform virtual key codes to [`Key`] values.
#[cfg(target_os = "windows")]
const KEY_MAPPINGS: &[(u16, Key)] = &[
  // Letter keys
  (0x00, Key::A),
  (0x01, Key::S),
  (0x02, Key::D),
  (0x03, Key::F),
  (0x04, Key::H),
  (0x05, Key::G),
  (0x06, Key::Z),
  (0x07, Key::X),
  (0x08, Key::C),
  (0x09, Key::V),
  (0x0B, Key::B),
  (0x0C, Key::Q),
  (0x0D, Key::W),
  (0x0E, Key::E),
  (0x0F, Key::R),
  (0x10, Key::Y),
  (0x11, Key::T),
  (0x1F, Key::O),
  (0x20, Key::U),
  (0x22, Key::I),
  (0x23, Key::P),
  (0x25, Key::L),
  (0x26, Key::J),
  (0x28, Key::K),
  (0x2D, Key::N),
  (0x2E, Key::M),
  // Numbers
  (0x12, Key::D1),
  (0x13, Key::D2),
  (0x14, Key::D3),
  (0x15, Key::D4),
  (0x16, Key::D6),
  (0x17, Key::D5),
  (0x19, Key::D9),
  (0x1A, Key::D7),
  (0x1C, Key::D8),
  (0x1D, Key::D0),
  // Function keys
  (0x7A, Key::F1),
  (0x78, Key::F2),
  (0x63, Key::F3),
  (0x76, Key::F4),
  (0x60, Key::F5),
  (0x61, Key::F6),
  (0x62, Key::F7),
  (0x64, Key::F8),
  (0x65, Key::F9),
  (0x6D, Key::F10),
  (0x67, Key::F11),
  (0x6F, Key::F12),
  // Modifier keys
  (0x37, Key::Cmd),
  (0x3A, Key::Alt),
  (0x3B, Key::Ctrl),
  (0x38, Key::Shift),
  // Special keys
  (0x31, Key::Space),
  (0x30, Key::Tab),
  (0x24, Key::Enter),
  (0x33, Key::Delete),
  (0x35, Key::Escape),
  // Arrow keys
  (0x7B, Key::Left),
  (0x7C, Key::Right),
  (0x7D, Key::Down),
  (0x7E, Key::Up),
  // Punctuation
  (0x18, Key::Equal),
  (0x1B, Key::Minus),
  (0x1E, Key::RightBracket),
  (0x21, Key::LeftBracket),
  (0x27, Key::Quote),
  (0x29, Key::Semicolon),
  (0x2A, Key::Backslash),
  (0x2B, Key::Comma),
  (0x2C, Key::Slash),
  (0x2F, Key::Period),
  (0x32, Key::Grave),
];

// TODO: Add complete macOS key code mappings.
#[cfg(target_os = "macos")]
const KEY_MAPPINGS: &[(i64, Key)] = &[
  // Letter keys
  (0x00, Key::A),
  (0x01, Key::S),
  (0x02, Key::D),
  (0x03, Key::F),
  (0x04, Key::H),
  (0x05, Key::G),
  (0x06, Key::Z),
  (0x07, Key::X),
  (0x08, Key::C),
  (0x09, Key::V),
  (0x0B, Key::B),
  (0x0C, Key::Q),
  (0x0D, Key::W),
  (0x0E, Key::E),
  (0x0F, Key::R),
  (0x10, Key::Y),
  (0x11, Key::T),
  (0x1F, Key::O),
  (0x20, Key::U),
  (0x22, Key::I),
  (0x23, Key::P),
  (0x25, Key::L),
  (0x26, Key::J),
  (0x28, Key::K),
  (0x2D, Key::N),
  (0x2E, Key::M),
  // Numbers
  (0x12, Key::D1),
  (0x13, Key::D2),
  (0x14, Key::D3),
  (0x15, Key::D4),
  (0x16, Key::D6),
  (0x17, Key::D5),
  (0x19, Key::D9),
  (0x1A, Key::D7),
  (0x1C, Key::D8),
  (0x1D, Key::D0),
  // Function keys
  (0x7A, Key::F1),
  (0x78, Key::F2),
  (0x63, Key::F3),
  (0x76, Key::F4),
  (0x60, Key::F5),
  (0x61, Key::F6),
  (0x62, Key::F7),
  (0x64, Key::F8),
  (0x65, Key::F9),
  (0x6D, Key::F10),
  (0x67, Key::F11),
  (0x6F, Key::F12),
  // Modifier keys
  (0x37, Key::Cmd),
  (0x3A, Key::Alt),
  (0x3B, Key::Ctrl),
  (0x38, Key::Shift),
  // Special keys
  (0x31, Key::Space),
  (0x30, Key::Tab),
  (0x24, Key::Enter),
  (0x33, Key::Delete),
  (0x35, Key::Escape),
  // Arrow keys
  (0x7B, Key::Left),
  (0x7C, Key::Right),
  (0x7D, Key::Down),
  (0x7E, Key::Up),
  // Punctuation
  (0x18, Key::Equal),
  (0x1B, Key::Minus),
  (0x1E, Key::RightBracket),
  (0x21, Key::LeftBracket),
  (0x27, Key::Quote),
  (0x29, Key::Semicolon),
  (0x2A, Key::Backslash),
  (0x2B, Key::Comma),
  (0x2C, Key::Slash),
  (0x2F, Key::Period),
  (0x32, Key::Grave),
];

impl From<KeyCode> for Key {
  fn from(keycode: KeyCode) -> Self {
    KEY_MAPPINGS
      .iter()
      .find_map(
        |(code, key)| if *code == keycode.0 { Some(*key) } else { None },
      )
      .unwrap_or(Key::Raw(keycode))
  }
}

impl From<Key> for KeyCode {
  fn from(key: Key) -> Self {
    match key {
      Key::Raw(keycode) => keycode,
      key => KEY_MAPPINGS
        .iter()
        .find(|(_, mapped_key)| *mapped_key == key)
        .map(|(code, _)| KeyCode(*code))
        .unwrap_or_else(|| {
          // Handle keys that aren't in the mapping yet
          match key {
            Key::F13 => todo!(),
            Key::F14 => todo!(),
            Key::F15 => todo!(),
            Key::F16 => todo!(),
            Key::F17 => todo!(),
            Key::F18 => todo!(),
            Key::F19 => todo!(),
            Key::F20 => todo!(),
            Key::F21 => todo!(),
            Key::F22 => todo!(),
            Key::F23 => todo!(),
            Key::F24 => todo!(),
            Key::LCmd => todo!(),
            Key::RCmd => todo!(),
            Key::LCtrl => todo!(),
            Key::RCtrl => todo!(),
            Key::LAlt => todo!(),
            Key::RAlt => todo!(),
            Key::LShift => todo!(),
            Key::RShift => todo!(),
            Key::LWin => todo!(),
            Key::RWin => todo!(),
            Key::Win => todo!(),
            Key::Return => todo!(),
            Key::Backspace => todo!(),
            Key::Home => todo!(),
            Key::End => todo!(),
            Key::PageUp => todo!(),
            Key::PageDown => todo!(),
            Key::Insert => todo!(),
            Key::Numpad0 => todo!(),
            Key::Numpad1 => todo!(),
            Key::Numpad2 => todo!(),
            Key::Numpad3 => todo!(),
            Key::Numpad4 => todo!(),
            Key::Numpad5 => todo!(),
            Key::Numpad6 => todo!(),
            Key::Numpad7 => todo!(),
            Key::Numpad8 => todo!(),
            Key::Numpad9 => todo!(),
            Key::NumpadAdd => todo!(),
            Key::NumpadSubtract => todo!(),
            Key::NumpadMultiply => todo!(),
            Key::NumpadDivide => todo!(),
            Key::NumpadDecimal => todo!(),
            Key::NumLock => todo!(),
            Key::ScrollLock => todo!(),
            Key::CapsLock => todo!(),
            Key::VolumeUp => todo!(),
            Key::VolumeDown => todo!(),
            Key::VolumeMute => todo!(),
            Key::MediaNextTrack => todo!(),
            Key::MediaPrevTrack => todo!(),
            Key::MediaStop => todo!(),
            Key::MediaPlayPause => todo!(),
            Key::PrintScreen => todo!(),
            Key::OemSemicolon => todo!(),
            Key::OemQuestion => todo!(),
            Key::OemTilde => todo!(),
            Key::OemOpenBrackets => todo!(),
            Key::OemPipe => todo!(),
            Key::OemCloseBrackets => todo!(),
            Key::OemQuotes => todo!(),
            Key::OemPlus => todo!(),
            Key::OemComma => todo!(),
            Key::OemMinus => todo!(),
            Key::OemPeriod => todo!(),
            _ => unreachable!(),
          }
        }),
    }
  }
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
      Key::Alt,
      Key::Ctrl,
      Key::Shift,
      Key::Space,
      Key::Tab,
      Key::Enter,
      Key::F1,
      Key::F12,
      Key::Left,
      Key::Right,
    ];

    // TODO: Iterate over 0..254 instead.
    for key in test_keys {
      let code: KeyCode = key.into();
      let key2: Key = code.into();
      assert_eq!(key, key2, "Roundtrip failed for key: {key:?}");
    }
  }
}
