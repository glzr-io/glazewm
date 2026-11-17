use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::Color;

/// Shades of the system accent color, from lightest to darkest.
///
/// # Platform-specific
///
/// - **Windows**: Mirrors the `AccentPalette` registry value, which backs
///   the `UIColorType::Accent*` values of the `UISettings` API.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccentPalette {
  pub accent_light3: Color,
  pub accent_light2: Color,
  pub accent_light1: Color,
  pub accent: Color,
  pub accent_dark1: Color,
  pub accent_dark2: Color,
  pub accent_dark3: Color,
}

impl AccentPalette {
  /// Gets the color of the given shade.
  #[must_use]
  pub fn color(&self, shade: AccentShade) -> &Color {
    match shade {
      AccentShade::AccentLight3 => &self.accent_light3,
      AccentShade::AccentLight2 => &self.accent_light2,
      AccentShade::AccentLight1 => &self.accent_light1,
      AccentShade::Accent => &self.accent,
      AccentShade::AccentDark1 => &self.accent_dark1,
      AccentShade::AccentDark2 => &self.accent_dark2,
      AccentShade::AccentDark3 => &self.accent_dark3,
    }
  }

  /// Decodes the Windows `AccentPalette` registry value.
  ///
  /// The value holds 8 colors of 4 bytes each (RGBA, alpha unused). The
  /// first 7 are the shades from lightest to darkest; the 8th is unused.
  pub fn from_registry_bytes(bytes: &[u8]) -> crate::Result<Self> {
    if bytes.len() < 28 {
      return Err(crate::Error::Platform(format!(
        "Invalid accent palette length: {}.",
        bytes.len()
      )));
    }

    let color = |index: usize| Color {
      r: bytes[index * 4],
      g: bytes[index * 4 + 1],
      b: bytes[index * 4 + 2],
      a: 255,
    };

    Ok(Self {
      accent_light3: color(0),
      accent_light2: color(1),
      accent_light1: color(2),
      accent: color(3),
      accent_dark1: color(4),
      accent_dark2: color(5),
      accent_dark3: color(6),
    })
  }
}

/// A shade of the system accent color (e.g. `accent_dark1`).
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccentShade {
  AccentLight3,
  AccentLight2,
  AccentLight1,
  Accent,
  AccentDark1,
  AccentDark2,
  AccentDark3,
}

impl AccentShade {
  /// Gets the name of the shade as used in config files.
  #[must_use]
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::AccentLight3 => "accent_light3",
      Self::AccentLight2 => "accent_light2",
      Self::AccentLight1 => "accent_light1",
      Self::Accent => "accent",
      Self::AccentDark1 => "accent_dark1",
      Self::AccentDark2 => "accent_dark2",
      Self::AccentDark3 => "accent_dark3",
    }
  }
}

impl FromStr for AccentShade {
  type Err = crate::ParseError;

  fn from_str(unparsed: &str) -> Result<Self, crate::ParseError> {
    [
      Self::AccentLight3,
      Self::AccentLight2,
      Self::AccentLight1,
      Self::Accent,
      Self::AccentDark1,
      Self::AccentDark2,
      Self::AccentDark3,
    ]
    .into_iter()
    .find(|shade| shade.as_str() == unparsed)
    .ok_or_else(|| crate::ParseError::AccentShade(unparsed.to_string()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decodes_registry_bytes() {
    let bytes = (0u8..32).collect::<Vec<_>>();
    let palette = AccentPalette::from_registry_bytes(&bytes).unwrap();

    assert_eq!(
      palette.accent_light3,
      Color {
        r: 0,
        g: 1,
        b: 2,
        a: 255
      }
    );
    assert_eq!(
      palette.accent,
      Color {
        r: 12,
        g: 13,
        b: 14,
        a: 255
      }
    );
    assert_eq!(
      palette.accent_dark3,
      Color {
        r: 24,
        g: 25,
        b: 26,
        a: 255
      }
    );
  }

  #[test]
  fn parses_shades() {
    assert_eq!(
      "accent".parse::<AccentShade>().unwrap(),
      AccentShade::Accent
    );
    assert_eq!(
      "accent_dark2".parse::<AccentShade>().unwrap(),
      AccentShade::AccentDark2
    );
    assert!("Accent".parse::<AccentShade>().is_err());
    assert!("#accent".parse::<AccentShade>().is_err());
  }

  #[test]
  fn rejects_short_registry_bytes() {
    assert!(AccentPalette::from_registry_bytes(&[0; 27]).is_err());
  }
}
