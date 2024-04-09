use std::str::FromStr;

use anyhow::bail;

#[derive(Clone, Debug, PartialEq)]
pub enum WindowState {
  Floating,
  Fullscreen,
  Maximized,
  Minimized,
  Tiling,
}

impl FromStr for WindowState {
  type Err = anyhow::Error;

  fn from_str(unparsed: &str) -> anyhow::Result<Self> {
    match unparsed {
      "floating" => Ok(WindowState::Floating),
      "fullscreen" => Ok(WindowState::Fullscreen),
      "maximized" => Ok(WindowState::Maximized),
      "minimized" => Ok(WindowState::Minimized),
      "tiling" => Ok(WindowState::Tiling),
      _ => bail!(
        "Not a valid window state '{}'. Must be one of the following: \
        'floating', 'fullscreen', 'maximized', 'minimized', or 'tiling'.",
        unparsed
      ),
    }
  }
}
