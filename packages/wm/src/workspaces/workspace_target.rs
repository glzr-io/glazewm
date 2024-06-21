use crate::common::Direction;

pub enum WorkspaceTarget {
  Name(String),
  Recent,
  Next,
  Previous,
  Direction(Direction),
}
