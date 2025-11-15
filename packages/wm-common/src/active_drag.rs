use serde::{Deserialize, Serialize};
use wm_platform::Rect;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActiveDrag {
  /// Whether the drag is a move or resize.
  pub operation: Option<ActiveDragOperation>,

  /// Whether the drag is from a tiling window.
  pub is_from_tiling: bool,

  /// Initial position when the drag started. Used to calculate movement
  /// distance.
  pub initial_position: Rect,
}

#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Serialize)]
pub enum ActiveDragOperation {
  Moving,
  Resizing,
}
