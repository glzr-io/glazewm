use std::str::FromStr;

/// A wrapper that indicates a value should be interpreted as a delta
/// (relative change).
#[derive(Debug, Clone, Copy)]
pub struct Delta<T> {
  pub inner: T,
  pub is_negative: bool,
}

impl<T: FromStr<Err = anyhow::Error>> Delta<T> {
  pub fn parse(s: &str) -> Result<Self, anyhow::Error> {
    let s = s.trim();

    let (raw, is_negative) = match s.chars().next() {
      Some('+') => (&s[1..], false),
      Some('-') => (&s[1..], true),
      _ => (s, false), // No sign means positive
    };

    if raw.is_empty() {
      return Err(anyhow::anyhow!("Empty value"));
    }

    let inner = T::from_str(raw)?;

    Ok(Self { inner, is_negative })
  }

  // /// Gets a reference to the inner value.
  // pub fn inner(&self) -> &T {
  //   &self.inner
  // }

  // /// Gets the sign of the delta.
  // pub fn is_negative(&self) -> bool {
  //   self.is_negative
  // }
}
