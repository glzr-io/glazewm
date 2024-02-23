use std::str::FromStr;

use regex::Regex;

pub struct LengthValue {
  pub amount: f32,
  pub unit: LengthUnit,
}

pub enum LengthUnit {
  Pixel,
  Percentage,
}

impl LengthValue {
  /// Parses a string containing a number followed by a unit (`px`, `%`).
  ///
  /// Example:
  /// ```
  /// LengthValue::from_str("100px") // { amount: 100.0, unit: LengthUnit::Pixel }
  /// ```
  fn from_str(unparsed: &str) -> Result<LengthValue> {
    let units_regex = Regex::new(r"(\d+)(%|ppt|px)?")?;

    let captures = units_regex.captures(unparsed)?;
    let amount = f32::from_str(&captures[1])?;

    let unit_str = captures.get(2).map_or("", |m| m.as_str());
    let unit = match unit_str {
      "px" | "" => Unit::Pixel,
      "%" => Unit::Percentage,
      _ => bail!("Not a valid unit '{}'.", unit_str),
    };

    Ok(LengthValue { amount, unit })
  }
}
