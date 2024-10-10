use crate::common::Direction;

pub enum WorkspaceTarget {
  Name(String),
  Recent,
  Next,
  Previous,
  NextOrder,
  PreviousOrder,
  Direction(Direction),
}
