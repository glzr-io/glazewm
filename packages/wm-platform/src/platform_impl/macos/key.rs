use crate::Key;

/// Converts a `Key` to its macOS key code.
pub(crate) fn key_to_macos_code(key: Key) -> Option<i64> {
  match key {
    // Letter keys
    Key::A => Some(0x00),
    Key::S => Some(0x01),
    Key::D => Some(0x02),
    Key::F => Some(0x03),
    Key::H => Some(0x04),
    Key::G => Some(0x05),
    Key::Z => Some(0x06),
    Key::X => Some(0x07),
    Key::C => Some(0x08),
    Key::V => Some(0x09),
    Key::B => Some(0x0B),
    Key::Q => Some(0x0C),
    Key::W => Some(0x0D),
    Key::E => Some(0x0E),
    Key::R => Some(0x0F),
    Key::Y => Some(0x10),
    Key::T => Some(0x11),
    Key::O => Some(0x1F),
    Key::U => Some(0x20),
    Key::I => Some(0x22),
    Key::P => Some(0x23),
    Key::L => Some(0x25),
    Key::J => Some(0x26),
    Key::K => Some(0x28),
    Key::N => Some(0x2D),
    Key::M => Some(0x2E),

    // Numbers
    Key::D1 => Some(0x12),
    Key::D2 => Some(0x13),
    Key::D3 => Some(0x14),
    Key::D4 => Some(0x15),
    Key::D6 => Some(0x16),
    Key::D5 => Some(0x17),
    Key::D9 => Some(0x19),
    Key::D7 => Some(0x1A),
    Key::D8 => Some(0x1C),
    Key::D0 => Some(0x1D),

    // Function keys
    Key::F1 => Some(0x7A),
    Key::F2 => Some(0x78),
    Key::F3 => Some(0x63),
    Key::F4 => Some(0x76),
    Key::F5 => Some(0x60),
    Key::F6 => Some(0x61),
    Key::F7 => Some(0x62),
    Key::F8 => Some(0x64),
    Key::F9 => Some(0x65),
    Key::F10 => Some(0x6D),
    Key::F11 => Some(0x67),
    Key::F12 => Some(0x6F),

    // Modifier keys
    Key::Cmd => Some(0x37),
    Key::Alt => Some(0x3A),
    Key::Ctrl => Some(0x3B),
    Key::Shift => Some(0x38),

    // Special keys
    Key::Space => Some(0x31),
    Key::Tab => Some(0x30),
    Key::Enter => Some(0x24),
    Key::Delete => Some(0x33),
    Key::Escape => Some(0x35),

    // Arrow keys
    Key::Left => Some(0x7B),
    Key::Right => Some(0x7C),
    Key::Down => Some(0x7D),
    Key::Up => Some(0x7E),

    // Punctuation
    Key::Equal => Some(0x18),
    Key::Minus => Some(0x1B),
    Key::RightBracket => Some(0x1E),
    Key::LeftBracket => Some(0x21),
    Key::Quote => Some(0x27),
    Key::Semicolon => Some(0x29),
    Key::Backslash => Some(0x2A),
    Key::Comma => Some(0x2B),
    Key::Slash => Some(0x2C),
    Key::Period => Some(0x2F),
    Key::Grave => Some(0x32),

    _ => None,
  }
}

/// Converts a macOS key code to a `Key`.
pub(crate) fn macos_code_to_key(code: i64) -> Option<Key> {
  match code {
    // Letter keys
    0x00 => Some(Key::A),
    0x01 => Some(Key::S),
    0x02 => Some(Key::D),
    0x03 => Some(Key::F),
    0x04 => Some(Key::H),
    0x05 => Some(Key::G),
    0x06 => Some(Key::Z),
    0x07 => Some(Key::X),
    0x08 => Some(Key::C),
    0x09 => Some(Key::V),
    0x0B => Some(Key::B),
    0x0C => Some(Key::Q),
    0x0D => Some(Key::W),
    0x0E => Some(Key::E),
    0x0F => Some(Key::R),
    0x10 => Some(Key::Y),
    0x11 => Some(Key::T),
    0x1F => Some(Key::O),
    0x20 => Some(Key::U),
    0x22 => Some(Key::I),
    0x23 => Some(Key::P),
    0x25 => Some(Key::L),
    0x26 => Some(Key::J),
    0x28 => Some(Key::K),
    0x2D => Some(Key::N),
    0x2E => Some(Key::M),

    // Numbers
    0x12 => Some(Key::D1),
    0x13 => Some(Key::D2),
    0x14 => Some(Key::D3),
    0x15 => Some(Key::D4),
    0x16 => Some(Key::D6),
    0x17 => Some(Key::D5),
    0x19 => Some(Key::D9),
    0x1A => Some(Key::D7),
    0x1C => Some(Key::D8),
    0x1D => Some(Key::D0),

    // Function keys
    0x7A => Some(Key::F1),
    0x78 => Some(Key::F2),
    0x63 => Some(Key::F3),
    0x76 => Some(Key::F4),
    0x60 => Some(Key::F5),
    0x61 => Some(Key::F6),
    0x62 => Some(Key::F7),
    0x64 => Some(Key::F8),
    0x65 => Some(Key::F9),
    0x6D => Some(Key::F10),
    0x67 => Some(Key::F11),
    0x6F => Some(Key::F12),

    // Modifier keys
    0x37 => Some(Key::Cmd),
    0x3A => Some(Key::Alt),
    0x3B => Some(Key::Ctrl),
    0x38 => Some(Key::Shift),

    // Special keys
    0x31 => Some(Key::Space),
    0x30 => Some(Key::Tab),
    0x24 => Some(Key::Enter),
    0x33 => Some(Key::Delete),
    0x35 => Some(Key::Escape),

    // Arrow keys
    0x7B => Some(Key::Left),
    0x7C => Some(Key::Right),
    0x7D => Some(Key::Down),
    0x7E => Some(Key::Up),

    // Punctuation
    0x18 => Some(Key::Equal),
    0x1B => Some(Key::Minus),
    0x1E => Some(Key::RightBracket),
    0x21 => Some(Key::LeftBracket),
    0x27 => Some(Key::Quote),
    0x29 => Some(Key::Semicolon),
    0x2A => Some(Key::Backslash),
    0x2B => Some(Key::Comma),
    0x2C => Some(Key::Slash),
    0x2F => Some(Key::Period),
    0x32 => Some(Key::Grave),

    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_key_to_macos_code() {
    assert_eq!(key_to_macos_code(Key::A), Some(0x00));
    assert_eq!(key_to_macos_code(Key::Cmd), Some(0x37));
    assert_eq!(key_to_macos_code(Key::F1), Some(0x7A));
    assert_eq!(key_to_macos_code(Key::Space), Some(0x31));
  }

  #[test]
  fn test_macos_code_to_key() {
    assert_eq!(macos_code_to_key(0x00), Some(Key::A));
    assert_eq!(macos_code_to_key(0x37), Some(Key::Cmd));
    assert_eq!(macos_code_to_key(0x7A), Some(Key::F1));
    assert_eq!(macos_code_to_key(0x31), Some(Key::Space));
  }

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

    for key in test_keys {
      if let Some(code) = key_to_macos_code(key) {
        assert_eq!(
          macos_code_to_key(code),
          Some(key),
          "Roundtrip failed for key: {:?}",
          key
        );
      }
    }
  }

  #[test]
  fn test_unknown_key_codes() {
    assert_eq!(macos_code_to_key(0xFFFF), None);
    assert_eq!(key_to_macos_code(Key::NumpadAdd), None); // Not mapped
  }
}
