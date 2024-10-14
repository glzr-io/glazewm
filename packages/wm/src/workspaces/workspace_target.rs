use crate::common::Direction;

pub enum WorkspaceTarget {
  Name(String),
  Recent,
  NextActive,
  PreviousActive,
  Next,
  Previous,
  Direction(Direction),
}
