use wm_common::Direction;

pub enum WorkspaceTarget {
  Name(String),
  Recent,
  NextActive,
  PreviousActive,
  Next,
  Previous,
  #[allow(dead_code)]
  Direction(Direction),
}
