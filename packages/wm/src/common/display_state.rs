/// Represents whether something is shown, hidden, or in an intermediary
/// state.
#[derive(Clone, Debug, PartialEq)]
pub enum DisplayState {
  Shown,
  Showing,
  Hidden,
  Hiding,
}
