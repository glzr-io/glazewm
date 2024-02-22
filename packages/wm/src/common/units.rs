use regex::Regex;
use std::str::FromStr;

pub enum Unit {
  Pixel,
  Percentage,
  None,
}

/// Parses a string containing a number followed by a unit (`px`, `%`).
///
/// Returns a tuple where the first element is a float, and the second
/// element is the unit as an enum value.
pub fn extract_units(amount_with_units: &str) -> (f32, Unit) {
  let units_regex = Regex::new(r"(\d+)(%|ppt|px)?").unwrap();

  let captures = units_regex.captures(amount_with_units).unwrap();
  let amount = f32::from_str(&captures[1]).unwrap_or(0.0);

  let unit = match captures.get(2).map_or("", |m| m.as_str()) {
    "px" => Unit::Pixel,
    "%" => Unit::Percentage,
    _ => Unit::None,
  };

  (amount, unit)
}
