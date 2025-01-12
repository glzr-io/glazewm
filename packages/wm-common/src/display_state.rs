use serde::{Deserialize, Serialize};

/// Represents whether something is shown, hidden, or in an intermediary
/// state.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplayState {
  Shown,
  Showing,
  Hidden,
  Hiding,
}
