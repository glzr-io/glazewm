use std::str::FromStr;

use serde::Serialize;

/// A wrapper that indicates a value should be interpreted as a delta
/// (relative change).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Delta<T> {
  pub inner: T,
  pub is_negative: bool,
}

impl<T: FromStr<Err = crate::ParseError>> FromStr for Delta<T> {
  type Err = crate::ParseError;

  fn from_str(unparsed: &str) -> Result<Self, crate::ParseError> {
    let unparsed = unparsed.trim();

    let (raw, is_negative) = match unparsed.chars().next() {
      Some('+') => (&unparsed[1..], false),
      Some('-') => (&unparsed[1..], true),
      // No sign is interpreted as positive.
      _ => (unparsed, false),
    };

    if raw.is_empty() {
      return Err(crate::ParseError::Delta(unparsed.to_string()));
    }

    let inner = T::from_str(raw)?;

    Ok(Self { inner, is_negative })
  }
}
