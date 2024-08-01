#[derive(Debug, Clone, Default)]
pub struct ActiveDrag {
  pub operation: Option<ActiveDragOperation>,
  pub is_from_tiling: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ActiveDragOperation {
  Moving,
  Resizing,
}
