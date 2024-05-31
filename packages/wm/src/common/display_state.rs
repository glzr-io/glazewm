use serde::Serialize;

/// Represents whether something is shown, hidden, or in an intermediary
/// state.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum DisplayState {
  Shown,
  Showing,
  Hidden,
  Hiding,
}
