use crate::common::Direction;

#[derive(Clone)]
pub enum WorkspaceTarget {
  Name(String),
  Recent,
  NextActive,
  PreviousActive,
  Next,
  Previous,
  Direction(Direction),
}
