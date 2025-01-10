use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ActiveDrag {
  pub operation: Option<ActiveDragOperation>,
  pub is_from_tiling: bool,
}

#[derive(Debug, Copy, Clone, Deserialize, PartialEq, Serialize)]
pub enum ActiveDragOperation {
  Moving,
  Resizing,
}
