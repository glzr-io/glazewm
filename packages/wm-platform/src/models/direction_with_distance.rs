use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

use crate::{Direction, LengthValue};

/// A direction paired with an optional movement distance (e.g. `right`,
/// `right,10px`, or `right,2%`).
///
/// When no distance is provided, the default move distance is applied.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DirectionWithDistance {
  pub direction: Direction,
  pub distance: Option<LengthValue>,
}

impl FromStr for DirectionWithDistance {
  type Err = crate::ParseError;

  /// Parses a string into a `DirectionWithDistance`. Accepts either a bare
  /// direction (e.g. `right`) or a direction followed by a pixel or
  /// percentage distance (e.g. `right,10px`, `right,2%`).
  ///
  /// Example:
  ///
  /// ```
  /// # use wm_platform::{Direction, DirectionWithDistance, LengthUnit, LengthValue};
  /// # use std::str::FromStr;
  /// let bare = DirectionWithDistance::from_str("right").unwrap();
  /// assert_eq!(bare.direction, Direction::Right);
  /// assert_eq!(bare.distance, None);
  ///
  /// let with_px = DirectionWithDistance::from_str("right,10px").unwrap();
  /// assert_eq!(with_px.direction, Direction::Right);
  /// assert_eq!(with_px.distance, Some(LengthValue { amount: 10.0, unit: LengthUnit::Pixel }));
  ///
  /// let with_pct = DirectionWithDistance::from_str("right,2%").unwrap();
  /// assert_eq!(with_pct.direction, Direction::Right);
  /// assert_eq!(with_pct.distance, Some(LengthValue { amount: 0.02, unit: LengthUnit::Percentage }));
  /// ```
  fn from_str(unparsed: &str) -> Result<Self, crate::ParseError> {
    let unparsed = unparsed.trim();
    let mut parts = unparsed.splitn(2, ',');

    let direction = parts
      .next()
      .ok_or_else(|| {
        crate::ParseError::DirectionWithDistance(unparsed.to_string())
      })?
      .parse::<Direction>()?;

    let distance = parts
      .next()
      .map(|s| s.trim().parse::<LengthValue>())
      .transpose()
      .map_err(|_| {
        crate::ParseError::DirectionWithDistance(unparsed.to_string())
      })?;

    Ok(Self {
      direction,
      distance,
    })
  }
}

/// Deserialize a `DirectionWithDistance` from a string (e.g. `"right"`,
/// `"right,10px"`, or `"right,2%"`).
impl<'de> Deserialize<'de> for DirectionWithDistance {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let str = String::deserialize(deserializer)?;
    Self::from_str(&str).map_err(serde::de::Error::custom)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! validate_error {
    ($input:expr, $pattern:pat) => {
      match DirectionWithDistance::from_str($input) {
        Err(e) if matches!(e, $pattern) => {}
        Err(e) => panic!(
          "input {:?}: expected error matching {}, got {:?}",
          $input,
          stringify!($pattern),
          e
        ),
        Ok(_) => panic!(
          "input {:?}: expected error matching {}, but received OK",
          $input,
          stringify!($pattern)
        ),
      }
    };
  }

  #[test]
  fn test_parse_invalid() {
    validate_error!("", crate::ParseError::Direction(_));
    validate_error!("invalid", crate::ParseError::Direction(_));
    validate_error!("left,", crate::ParseError::DirectionWithDistance(_));
    validate_error!(
      "right,m",
      crate::ParseError::DirectionWithDistance(_)
    );
  }
}
